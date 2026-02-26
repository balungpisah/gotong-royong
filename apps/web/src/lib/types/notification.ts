/**
 * Notification domain types — in-app notification feed.
 */

/** All notification type variants. */
export type NotificationType =
	| 'phase_change'
	| 'vote_open'
	| 'evidence_needed'
	| 'diff_proposed'
	| 'mention'
	| 'role_assigned'
	| 'system';

/**
 * An in-app notification.
 */
export interface AppNotification {
	notification_id: string;
	type: NotificationType;
	title: string;
	body: string;
	witness_id?: string;
	target_path?: string;
	read: boolean;
	created_at: string;
}
