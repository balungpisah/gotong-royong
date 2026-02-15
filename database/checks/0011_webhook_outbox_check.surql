-- 0011_webhook_outbox_check
INFO FOR TABLE webhook_outbox_event;
INFO FOR TABLE webhook_delivery_log;
SELECT count() AS webhook_outbox_event_rows FROM webhook_outbox_event;
SELECT count() AS webhook_delivery_log_rows FROM webhook_delivery_log;
