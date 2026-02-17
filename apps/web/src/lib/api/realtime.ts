import { browser } from '$app/environment';

export type TransportMode = 'websocket' | 'sse' | 'polling' | 'disconnected';

export interface RealtimeStatus {
	mode: TransportMode;
	isConnected: boolean;
	lastError: string | null;
	lastTransitionAt: number;
}

export interface RealtimeCursor {
	sinceCreatedAtMs?: number;
	sinceMessageId?: string;
}

export type RealtimeQueryValue = string | number | boolean | null | undefined;

export type RealtimeQueryParams = Record<
	string,
	RealtimeQueryValue | ReadonlyArray<RealtimeQueryValue>
>;

type MaybePromise<T> = T | Promise<T>;
type Listener<TEvent> = (event: TEvent) => void;
type StatusListener = (status: RealtimeStatus) => void;

export interface RealtimeTransportConfig<TEvent, TPollItem = TEvent> {
	websocketUrl: string;
	sseUrl: string;
	pollingUrl: string;
	getAuthToken?: () => MaybePromise<string | null>;
	parseWebsocketMessage?: (payload: unknown) => TEvent | null;
	parseSseMessage?: (payload: unknown, eventType?: string) => TEvent | null;
	parsePollItem?: (payload: TPollItem) => TEvent | null;
	cursorFromEvent?: (event: TEvent) => RealtimeCursor | null;
	buildPollQuery?: (cursor: RealtimeCursor | null) => RealtimeQueryParams | undefined;
	onEvent?: (event: TEvent) => void;
	onDegradedToPolling?: (status: RealtimeStatus) => void;
	pollingIntervalMs?: number;
	reconnectIntervalMs?: number;
	wsConnectTimeoutMs?: number;
	sseConnectTimeoutMs?: number;
	includeTokenInQuery?: boolean;
	tokenQueryParam?: string;
	fetchFn?: typeof fetch;
}

const DEFAULT_POLLING_INTERVAL_MS = 5_000;
const DEFAULT_RECONNECT_INTERVAL_MS = 30_000;
const DEFAULT_WS_CONNECT_TIMEOUT_MS = 5_000;
const DEFAULT_SSE_CONNECT_TIMEOUT_MS = 5_000;
const DEFAULT_TOKEN_QUERY_PARAM = 'access_token';

const baseStatus = (): RealtimeStatus => ({
	mode: 'disconnected',
	isConnected: false,
	lastError: null,
	lastTransitionAt: Date.now()
});

const isRecord = (value: unknown): value is Record<string, unknown> =>
	typeof value === 'object' && value !== null;

const safeJsonParse = (payload: string): unknown => {
	try {
		return JSON.parse(payload);
	} catch {
		return payload;
	}
};

const appendQuery = (url: URL, query?: RealtimeQueryParams) => {
	if (!query) {
		return;
	}

	for (const [key, rawValue] of Object.entries(query)) {
		const values = Array.isArray(rawValue) ? rawValue : [rawValue];
		for (const value of values) {
			if (value === null || value === undefined) {
				continue;
			}
			url.searchParams.append(key, String(value));
		}
	}
};

const withQuery = (rawUrl: string, query?: RealtimeQueryParams) => {
	const url = new URL(rawUrl, 'http://localhost');
	appendQuery(url, query);
	if (url.origin === 'http://localhost') {
		return `${url.pathname}${url.search}${url.hash}`;
	}
	return url.toString();
};

const defaultEnvelopeParser = <TEvent>(payload: unknown): TEvent | null => {
	if (isRecord(payload) && payload.event_type === 'message' && 'message' in payload) {
		return payload.message as TEvent;
	}
	return payload as TEvent;
};

export class RealtimeTransportManager<TEvent, TPollItem = TEvent> {
	private readonly listeners = new Set<Listener<TEvent>>();
	private readonly statusListeners = new Set<StatusListener>();
	private readonly fetchFn: typeof fetch;
	private status: RealtimeStatus = baseStatus();
	private ws: WebSocket | null = null;
	private sse: EventSource | null = null;
	private pollTimer: ReturnType<typeof setTimeout> | null = null;
	private cursor: RealtimeCursor | null = null;
	private lastUpgradeAttemptAt = 0;
	private isStopped = true;

	public constructor(private readonly config: RealtimeTransportConfig<TEvent, TPollItem>) {
		if (config.fetchFn) {
			this.fetchFn = config.fetchFn;
		} else if (typeof globalThis.fetch === 'function') {
			this.fetchFn = globalThis.fetch.bind(globalThis);
		} else {
			this.fetchFn = (() => {
				throw new Error('Fetch API is unavailable for polling fallback.');
			}) as typeof fetch;
		}
	}

	public getStatus = () => this.status;

	public subscribe = (listener: Listener<TEvent>) => {
		this.listeners.add(listener);
		return () => this.listeners.delete(listener);
	};

	public subscribeStatus = (listener: StatusListener) => {
		this.statusListeners.add(listener);
		listener(this.status);
		return () => this.statusListeners.delete(listener);
	};

	public start = async () => {
		if (!browser || !this.isStopped) {
			return;
		}

		this.isStopped = false;
		this.lastUpgradeAttemptAt = 0;
		await this.connectWebSocketOrFallback();
	};

	public stop = () => {
		this.isStopped = true;
		this.lastUpgradeAttemptAt = 0;
		this.clearTransports();
		this.updateStatus({
			mode: 'disconnected',
			isConnected: false,
			lastError: null
		});
	};

	private updateStatus = (next: Partial<RealtimeStatus>) => {
		const previousMode = this.status.mode;
		this.status = {
			...this.status,
			...next,
			lastTransitionAt: Date.now()
		};
		for (const listener of this.statusListeners) {
			listener(this.status);
		}
		if (this.status.mode === 'polling' && previousMode !== 'polling') {
			this.config.onDegradedToPolling?.(this.status);
		}
	};

	private clearTransports = () => {
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}
		if (this.sse) {
			this.sse.close();
			this.sse = null;
		}
		if (this.pollTimer) {
			clearTimeout(this.pollTimer);
			this.pollTimer = null;
		}
	};

	private emitEvent = (event: TEvent) => {
		this.config.onEvent?.(event);
		for (const listener of this.listeners) {
			listener(event);
		}
		const nextCursor = this.config.cursorFromEvent?.(event) ?? null;
		if (nextCursor) {
			this.cursor = nextCursor;
		}
	};

	private resolveToken = async () => this.config.getAuthToken?.() ?? null;

	private withToken = (url: string, token: string | null, extraQuery?: RealtimeQueryParams) => {
		const query: RealtimeQueryParams = { ...(extraQuery ?? {}) };
		if (token && this.config.includeTokenInQuery) {
			query[this.config.tokenQueryParam ?? DEFAULT_TOKEN_QUERY_PARAM] = token;
		}
		return withQuery(url, query);
	};

	private connectWebSocketOrFallback = async () => {
		try {
			await this.connectWebSocket();
		} catch (error) {
			if (this.isStopped) {
				return;
			}
			this.updateStatus({
				mode: 'disconnected',
				isConnected: false,
				lastError: error instanceof Error ? error.message : 'WebSocket connection failed'
			});
			await this.connectSseOrFallback();
		}
	};

	private connectSseOrFallback = async () => {
		try {
			await this.connectSse();
		} catch (error) {
			if (this.isStopped) {
				return;
			}
			this.updateStatus({
				mode: 'disconnected',
				isConnected: false,
				lastError: error instanceof Error ? error.message : 'SSE connection failed'
			});
			this.startPolling();
		}
	};

	private connectWebSocket = async () => {
		const token = await this.resolveToken();
		const wsUrl = this.withToken(this.config.websocketUrl, token);
		this.updateStatus({ mode: 'websocket', isConnected: false, lastError: null });

		const socket = new WebSocket(wsUrl);
		this.ws = socket;

		try {
			await new Promise<void>((resolve, reject) => {
				const timeout = setTimeout(() => {
					socket.close();
					reject(new Error('WebSocket connection timed out'));
				}, this.config.wsConnectTimeoutMs ?? DEFAULT_WS_CONNECT_TIMEOUT_MS);

				socket.addEventListener(
					'open',
					() => {
						clearTimeout(timeout);
						resolve();
					},
					{ once: true }
				);

				socket.addEventListener(
					'error',
					() => {
						clearTimeout(timeout);
						reject(new Error('WebSocket connection error'));
					},
					{ once: true }
				);
			});
		} catch (error) {
			socket.close();
			if (this.ws === socket) {
				this.ws = null;
			}
			throw error;
		}

		if (this.isStopped) {
			socket.close();
			if (this.ws === socket) {
				this.ws = null;
			}
			return;
		}

		this.updateStatus({ mode: 'websocket', isConnected: true, lastError: null });

		socket.addEventListener('message', (event) => {
			const payload = typeof event.data === 'string' ? safeJsonParse(event.data) : event.data;
			const parsed =
				this.config.parseWebsocketMessage?.(payload) ?? defaultEnvelopeParser<TEvent>(payload);
			if (parsed) {
				this.emitEvent(parsed);
			}
		});

		socket.addEventListener('close', () => {
			if (this.isStopped) {
				return;
			}
			this.connectSseOrFallback().catch(() => {
				this.startPolling();
			});
		});
	};

	private connectSse = async () => {
		const token = await this.resolveToken();
		const sseUrl = this.withToken(this.config.sseUrl, token);
		this.updateStatus({ mode: 'sse', isConnected: false, lastError: null });

		const source = new EventSource(sseUrl, { withCredentials: true });
		this.sse = source;

		try {
			await new Promise<void>((resolve, reject) => {
				const timeout = setTimeout(() => {
					source.close();
					reject(new Error('SSE connection timed out'));
				}, this.config.sseConnectTimeoutMs ?? DEFAULT_SSE_CONNECT_TIMEOUT_MS);

				source.addEventListener(
					'open',
					() => {
						clearTimeout(timeout);
						resolve();
					},
					{ once: true }
				);

				source.addEventListener(
					'error',
					() => {
						clearTimeout(timeout);
						reject(new Error('SSE connection error'));
					},
					{ once: true }
				);
			});
		} catch (error) {
			source.close();
			if (this.sse === source) {
				this.sse = null;
			}
			throw error;
		}

		if (this.isStopped) {
			source.close();
			if (this.sse === source) {
				this.sse = null;
			}
			return;
		}

		this.updateStatus({ mode: 'sse', isConnected: true, lastError: null });

		source.addEventListener('message', (event) => {
			const payload = safeJsonParse(event.data);
			const parsed =
				this.config.parseSseMessage?.(payload, event.type) ??
				defaultEnvelopeParser<TEvent>(payload);
			if (parsed) {
				this.emitEvent(parsed);
			}
		});

		source.addEventListener('error', () => {
			if (this.isStopped) {
				return;
			}
			source.close();
			this.startPolling();
		});
	};

	private startPolling = () => {
		if (this.isStopped) {
			return;
		}

		if (this.sse) {
			this.sse.close();
			this.sse = null;
		}
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}

		this.lastUpgradeAttemptAt = 0;
		this.updateStatus({ mode: 'polling', isConnected: true, lastError: null });
		this.schedulePoll(0);
	};

	private tryUpgradeFromPolling = async () => {
		if (this.isStopped || this.status.mode !== 'polling') {
			return;
		}

		const now = Date.now();
		const reconnectIntervalMs = this.config.reconnectIntervalMs ?? DEFAULT_RECONNECT_INTERVAL_MS;
		if (now - this.lastUpgradeAttemptAt < reconnectIntervalMs) {
			return;
		}

		this.lastUpgradeAttemptAt = now;

		try {
			await this.connectWebSocket();
			return;
		} catch {
			if (this.ws) {
				this.ws.close();
				this.ws = null;
			}
		}

		try {
			await this.connectSse();
		} catch {
			if (this.sse) {
				this.sse.close();
				this.sse = null;
			}
			this.updateStatus({
				mode: 'polling',
				isConnected: true,
				lastError: 'Realtime transport is degraded to polling mode'
			});
		}
	};

	private schedulePoll = (delayMs: number) => {
		if (this.isStopped) {
			return;
		}

		if (this.pollTimer) {
			clearTimeout(this.pollTimer);
		}

		this.pollTimer = setTimeout(async () => {
			try {
				await this.pollOnce();
				this.updateStatus({ mode: 'polling', isConnected: true, lastError: null });
				await this.tryUpgradeFromPolling();
			} catch (error) {
				this.updateStatus({
					mode: 'polling',
					isConnected: false,
					lastError: error instanceof Error ? error.message : 'Polling request failed'
				});
			} finally {
				if (!this.isStopped && this.status.mode === 'polling') {
					this.schedulePoll(this.config.pollingIntervalMs ?? DEFAULT_POLLING_INTERVAL_MS);
				}
			}
		}, delayMs);
	};

	private pollOnce = async () => {
		const token = await this.resolveToken();
		const query = this.config.buildPollQuery?.(this.cursor) ?? undefined;
		const pollUrl = this.withToken(this.config.pollingUrl, null, query);
		const headers = new Headers({ accept: 'application/json' });
		if (token) {
			headers.set('authorization', `Bearer ${token}`);
		}

		const response = await this.fetchFn(pollUrl, {
			method: 'GET',
			headers,
			credentials: 'include'
		});

		if (!response.ok) {
			throw new Error(`Polling failed with status ${response.status}`);
		}

		const payload = (await response.json()) as unknown;
		if (!Array.isArray(payload)) {
			throw new Error('Polling response must be an array payload');
		}

		for (const rawItem of payload) {
			const parsed =
				this.config.parsePollItem?.(rawItem as TPollItem) ?? defaultEnvelopeParser<TEvent>(rawItem);
			if (parsed) {
				this.emitEvent(parsed);
			}
		}
	};
}

export const createDegradedPollingHook = <TEvent, TPollItem = TEvent>(
	manager: RealtimeTransportManager<TEvent, TPollItem>,
	onPolling: (status: RealtimeStatus) => void
) =>
	manager.subscribeStatus((status) => {
		if (status.mode === 'polling') {
			onPolling(status);
		}
	});
