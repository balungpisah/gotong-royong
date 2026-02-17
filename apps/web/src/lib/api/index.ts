export type ApiMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS';

export type ApiQueryValue = string | number | boolean | null | undefined;

export type ApiQueryParams = Record<string, ApiQueryValue | ReadonlyArray<ApiQueryValue>>;

export type ApiResponseType = 'json' | 'text' | 'void';

export interface ApiRetryPolicy {
	maxAttempts: number;
	baseDelayMs: number;
	maxDelayMs: number;
	retryableMethods: ReadonlyArray<ApiMethod>;
	retryableStatusCodes: ReadonlyArray<number>;
}

export type ApiAuthMode =
	| {
			type: 'cookie';
			credentials?: RequestCredentials;
	  }
	| {
			type: 'bearer';
			token: string;
			credentials?: RequestCredentials;
	  }
	| {
			type: 'none';
			credentials?: RequestCredentials;
	  };

export interface ApiRequestOptions<TBody = unknown> {
	method?: ApiMethod;
	headers?: HeadersInit;
	query?: ApiQueryParams;
	body?: TBody;
	auth?: ApiAuthMode;
	retry?: Partial<ApiRetryPolicy>;
	timeoutMs?: number;
	signal?: AbortSignal;
	responseType?: ApiResponseType;
}

export interface ApiClientConfig {
	baseUrl?: string;
	timeoutMs?: number;
	defaultHeaders?: HeadersInit;
	retryPolicy?: Partial<ApiRetryPolicy>;
	auth?: ApiAuthMode;
	fetchFn?: typeof fetch;
}

export interface ApiErrorDetails {
	code: string;
	message: string;
	details?: unknown;
}

export class ApiClientError extends Error {
	public readonly name = 'ApiClientError';

	public constructor(
		public readonly method: ApiMethod,
		public readonly url: string,
		public readonly status: number,
		public readonly code: string,
		message: string,
		public readonly details?: unknown,
		public readonly requestId?: string | null,
		public readonly correlationId?: string | null,
		public readonly cause?: unknown
	) {
		super(message);
	}
}

export const isApiClientError = (error: unknown): error is ApiClientError =>
	error instanceof ApiClientError;

export interface ApiClient {
	request<TResponse, TBody = unknown>(
		path: string,
		options?: ApiRequestOptions<TBody>
	): Promise<TResponse>;
	get<TResponse>(
		path: string,
		options?: Omit<ApiRequestOptions<never>, 'method' | 'body'>
	): Promise<TResponse>;
	post<TResponse, TBody = unknown>(
		path: string,
		options?: Omit<ApiRequestOptions<TBody>, 'method'>
	): Promise<TResponse>;
	put<TResponse, TBody = unknown>(
		path: string,
		options?: Omit<ApiRequestOptions<TBody>, 'method'>
	): Promise<TResponse>;
	patch<TResponse, TBody = unknown>(
		path: string,
		options?: Omit<ApiRequestOptions<TBody>, 'method'>
	): Promise<TResponse>;
	delete<TResponse, TBody = unknown>(
		path: string,
		options?: Omit<ApiRequestOptions<TBody>, 'method'>
	): Promise<TResponse>;
}

const DEFAULT_BASE_URL = '/v1';
const DEFAULT_TIMEOUT_MS = 10_000;
const DEFAULT_RETRY_POLICY: ApiRetryPolicy = {
	maxAttempts: 2,
	baseDelayMs: 250,
	maxDelayMs: 2_000,
	retryableMethods: ['GET', 'HEAD', 'OPTIONS'],
	retryableStatusCodes: [429, 502, 503, 504]
};

const DEFAULT_AUTH_MODE: ApiAuthMode = {
	type: 'cookie',
	credentials: 'include'
};

const RETRY_AFTER_HEADER = 'retry-after';
const REQUEST_ID_HEADER = 'x-request-id';
const CORRELATION_ID_HEADER = 'x-correlation-id';

class InvalidJsonResponseError extends Error {
	public readonly name = 'InvalidJsonResponseError';

	public constructor(
		public readonly rawBody: string,
		public readonly cause?: unknown
	) {
		super('Response body is not valid JSON');
	}
}

const isBodyInit = (value: unknown): value is BodyInit =>
	typeof value === 'string' ||
	value instanceof Blob ||
	value instanceof FormData ||
	value instanceof URLSearchParams ||
	value instanceof ArrayBuffer ||
	ArrayBuffer.isView(value);

const mergeRetryPolicy = (
	basePolicy: ApiRetryPolicy,
	override?: Partial<ApiRetryPolicy>
): ApiRetryPolicy => ({
	maxAttempts: Math.max(1, override?.maxAttempts ?? basePolicy.maxAttempts),
	baseDelayMs: Math.max(10, override?.baseDelayMs ?? basePolicy.baseDelayMs),
	maxDelayMs: Math.max(10, override?.maxDelayMs ?? basePolicy.maxDelayMs),
	retryableMethods: override?.retryableMethods ?? basePolicy.retryableMethods,
	retryableStatusCodes: override?.retryableStatusCodes ?? basePolicy.retryableStatusCodes
});

const mergeSignals = (...signals: Array<AbortSignal | undefined>) => {
	const activeSignals = signals.filter((signal): signal is AbortSignal => Boolean(signal));
	if (activeSignals.length === 0) {
		return undefined;
	}

	const controller = new AbortController();
	const abort = (reason?: unknown) => {
		if (!controller.signal.aborted) {
			controller.abort(reason);
		}
	};

	for (const signal of activeSignals) {
		if (signal.aborted) {
			abort(signal.reason);
			break;
		}
		signal.addEventListener('abort', () => abort(signal.reason), { once: true });
	}

	return controller.signal;
};

const appendQuery = (url: URL, query?: ApiQueryParams) => {
	if (!query) {
		return;
	}

	for (const [key, rawValue] of Object.entries(query)) {
		const values = Array.isArray(rawValue) ? rawValue : [rawValue];
		for (const value of values) {
			if (value === undefined || value === null) {
				continue;
			}
			url.searchParams.append(key, String(value));
		}
	}
};

const buildRequestUrl = (baseUrl: string, path: string, query?: ApiQueryParams) => {
	if (/^https?:\/\//i.test(path)) {
		const directUrl = new URL(path);
		appendQuery(directUrl, query);
		return directUrl.toString();
	}

	const normalizedPath = path.startsWith('/') ? path : `/${path}`;
	const normalizedBase = baseUrl.endsWith('/') ? baseUrl.slice(0, -1) : baseUrl;
	const rawUrl = `${normalizedBase}${normalizedPath}`;
	const url = new URL(rawUrl, 'http://localhost');
	appendQuery(url, query);

	if (url.origin === 'http://localhost') {
		return `${url.pathname}${url.search}${url.hash}`;
	}

	return url.toString();
};

const redactUrlForLogs = (url: string) => {
	try {
		const parsed = new URL(url, 'http://localhost');
		return `${parsed.pathname}${parsed.hash}`;
	} catch {
		return url.split('?')[0] ?? url;
	}
};

const parseRetryAfterHeader = (value: string | null) => {
	if (!value) {
		return undefined;
	}

	const seconds = Number(value);
	if (!Number.isNaN(seconds) && seconds >= 0) {
		return seconds * 1_000;
	}

	const retryAt = Date.parse(value);
	if (!Number.isNaN(retryAt)) {
		return Math.max(0, retryAt - Date.now());
	}

	return undefined;
};

const sleep = async (durationMs: number, signal?: AbortSignal) =>
	new Promise<void>((resolve, reject) => {
		if (signal?.aborted) {
			if (signal.reason instanceof Error) {
				reject(signal.reason);
				return;
			}
			const abortError = new Error('Sleep aborted');
			abortError.name = 'AbortError';
			reject(abortError);
			return;
		}

		const timeout = setTimeout(() => resolve(), durationMs);
		if (!signal) {
			return;
		}

		signal.addEventListener(
			'abort',
			() => {
				clearTimeout(timeout);
				if (signal.reason instanceof Error) {
					reject(signal.reason);
					return;
				}
				const abortError = new Error('Sleep aborted');
				abortError.name = 'AbortError';
				reject(abortError);
			},
			{ once: true }
		);
	});

const isRetryableMethod = (method: ApiMethod, policy: ApiRetryPolicy) =>
	policy.retryableMethods.includes(method);

const shouldRetryStatus = (status: number, policy: ApiRetryPolicy) =>
	policy.retryableStatusCodes.includes(status);

const computeRetryDelay = (attempt: number, response: Response | null, policy: ApiRetryPolicy) => {
	const fromHeader = parseRetryAfterHeader(response?.headers.get(RETRY_AFTER_HEADER) ?? null);
	if (fromHeader !== undefined) {
		return Math.min(policy.maxDelayMs, Math.max(0, fromHeader));
	}

	const exponential = Math.min(policy.maxDelayMs, policy.baseDelayMs * 2 ** (attempt - 1));
	const jitter = Math.floor(Math.random() * 100);
	return exponential + jitter;
};

const parseResponseBody = async <T>(response: Response, responseType: ApiResponseType) => {
	if (responseType === 'void' || response.status === 204 || response.status === 205) {
		return undefined as T;
	}

	if (responseType === 'text') {
		return (await response.text()) as T;
	}

	const raw = await response.text();
	if (!raw) {
		return undefined as T;
	}

	try {
		return JSON.parse(raw) as T;
	} catch (error) {
		throw new InvalidJsonResponseError(raw, error);
	}
};

const parseErrorResponseBody = async (response: Response): Promise<unknown> => {
	const raw = await response.text();
	if (!raw) {
		return undefined;
	}

	try {
		return JSON.parse(raw) as unknown;
	} catch {
		return raw;
	}
};

const normalizeErrorEnvelope = (body: unknown, fallbackMessage: string): ApiErrorDetails => {
	if (!body || typeof body !== 'object') {
		return {
			code: 'unknown_error',
			message: fallbackMessage,
			details: body
		};
	}

	const envelope = body as Record<string, unknown>;
	const nestedError = envelope.error;

	if (nestedError && typeof nestedError === 'object') {
		const nested = nestedError as Record<string, unknown>;
		return {
			code: typeof nested.code === 'string' ? nested.code : 'unknown_error',
			message: typeof nested.message === 'string' ? nested.message : fallbackMessage,
			details: nested.details
		};
	}

	if (typeof nestedError === 'string') {
		return {
			code: typeof envelope.code === 'string' ? envelope.code : 'unknown_error',
			message: nestedError,
			details: envelope.details
		};
	}

	return {
		code: typeof envelope.code === 'string' ? envelope.code : 'unknown_error',
		message: typeof envelope.message === 'string' ? envelope.message : fallbackMessage,
		details: envelope.details ?? body
	};
};

const buildRequestBody = (body: unknown, headers: Headers) => {
	if (body === undefined || body === null) {
		return undefined;
	}

	if (isBodyInit(body)) {
		return body;
	}

	if (!headers.has('content-type')) {
		headers.set('content-type', 'application/json');
	}
	return JSON.stringify(body);
};

const resolveAuthMode = (client: ApiClientConfig, request?: ApiAuthMode) =>
	request ?? client.auth ?? DEFAULT_AUTH_MODE;

const resolveCredentials = (auth: ApiAuthMode): RequestCredentials => {
	if (auth.type === 'cookie') {
		return auth.credentials ?? 'include';
	}
	return auth.credentials ?? 'omit';
};

const resolveFetchFn = (fetchFn?: typeof fetch) => {
	if (fetchFn) {
		return fetchFn;
	}
	if (typeof globalThis.fetch === 'function') {
		return globalThis.fetch.bind(globalThis);
	}
	throw new Error('Fetch API is not available. Provide fetchFn in ApiClientConfig.');
};

export const createApiClient = (config: ApiClientConfig = {}): ApiClient => {
	const baseUrl = config.baseUrl ?? DEFAULT_BASE_URL;
	const defaultTimeoutMs = config.timeoutMs ?? DEFAULT_TIMEOUT_MS;
	const defaultRetryPolicy = mergeRetryPolicy(DEFAULT_RETRY_POLICY, config.retryPolicy);
	const fetchFn = resolveFetchFn(config.fetchFn);
	const configHeaders = new Headers(config.defaultHeaders);
	if (!configHeaders.has('accept')) {
		configHeaders.set('accept', 'application/json');
	}

	const request = async <TResponse, TBody = unknown>(
		path: string,
		options: ApiRequestOptions<TBody> = {}
	) => {
		const method = options.method ?? 'GET';
		const responseType = options.responseType ?? 'json';
		const retryPolicy = mergeRetryPolicy(defaultRetryPolicy, options.retry);
		const maxAttempts = isRetryableMethod(method, retryPolicy) ? retryPolicy.maxAttempts : 1;
		const authMode = resolveAuthMode(config, options.auth);
		const url = buildRequestUrl(baseUrl, path, options.query);
		const safeUrl = redactUrlForLogs(url);
		let lastError: ApiClientError | null = null;

		for (let attempt = 1; attempt <= maxAttempts; attempt++) {
			const headers = new Headers(configHeaders);
			if (options.headers) {
				const requestHeaders = new Headers(options.headers);
				for (const [key, value] of requestHeaders.entries()) {
					headers.set(key, value);
				}
			}

			if (authMode.type === 'bearer') {
				headers.set('authorization', `Bearer ${authMode.token}`);
			}

			const timeoutController = new AbortController();
			let timedOut = false;
			const timeoutMs = options.timeoutMs ?? defaultTimeoutMs;
			const timeoutId = setTimeout(() => {
				timedOut = true;
				timeoutController.abort();
			}, timeoutMs);

			try {
				const response = await fetchFn(url, {
					method,
					headers,
					body: buildRequestBody(options.body, headers),
					credentials: resolveCredentials(authMode),
					signal: mergeSignals(options.signal, timeoutController.signal)
				});

				if (response.ok) {
					clearTimeout(timeoutId);
					try {
						return await parseResponseBody<TResponse>(response, responseType);
					} catch (error) {
						if (error instanceof InvalidJsonResponseError) {
							throw new ApiClientError(
								method,
								safeUrl,
								response.status,
								'invalid_json_response',
								error.message,
								{ rawBody: error.rawBody },
								response.headers.get(REQUEST_ID_HEADER),
								response.headers.get(CORRELATION_ID_HEADER),
								error.cause
							);
						}
						throw error;
					}
				}

				let errorBody: unknown = undefined;
				try {
					errorBody = await parseErrorResponseBody(response);
				} catch {
					errorBody = undefined;
				}

				const normalized = normalizeErrorEnvelope(
					errorBody,
					`${method} ${safeUrl} failed with status ${response.status}`
				);

				const error = new ApiClientError(
					method,
					safeUrl,
					response.status,
					normalized.code,
					normalized.message,
					normalized.details,
					response.headers.get(REQUEST_ID_HEADER),
					response.headers.get(CORRELATION_ID_HEADER)
				);
				lastError = error;

				if (attempt < maxAttempts && shouldRetryStatus(response.status, retryPolicy)) {
					const retryDelay = computeRetryDelay(attempt, response, retryPolicy);
					await sleep(retryDelay, options.signal);
					continue;
				}

				throw error;
			} catch (error) {
				clearTimeout(timeoutId);
				if (error instanceof ApiClientError) {
					throw error;
				}

				const abortedByCaller = options.signal?.aborted;
				const isAbortError =
					error instanceof Error && (error.name === 'AbortError' || error.name === 'TimeoutError');

				if (abortedByCaller && (isAbortError || error === options.signal?.reason)) {
					throw error;
				}

				const networkError = new ApiClientError(
					method,
					safeUrl,
					0,
					timedOut ? 'timeout_error' : 'network_error',
					timedOut
						? `Request timed out after ${timeoutMs}ms`
						: `Network request failed for ${method} ${safeUrl}`,
					undefined,
					null,
					null,
					error
				);
				lastError = networkError;

				if (attempt < maxAttempts) {
					const retryDelay = computeRetryDelay(attempt, null, retryPolicy);
					await sleep(retryDelay, options.signal);
					continue;
				}

				throw networkError;
			} finally {
				clearTimeout(timeoutId);
			}
		}

		throw (
			lastError ??
			new ApiClientError(
				method,
				safeUrl,
				0,
				'unknown_error',
				`Failed to execute request ${method} ${safeUrl}`
			)
		);
	};

	return {
		request,
		get: (path, options) => request(path, { ...options, method: 'GET' }),
		post: (path, options) => request(path, { ...options, method: 'POST' }),
		put: (path, options) => request(path, { ...options, method: 'PUT' }),
		patch: (path, options) => request(path, { ...options, method: 'PATCH' }),
		delete: (path, options) => request(path, { ...options, method: 'DELETE' })
	};
};

export const apiClient = createApiClient();
