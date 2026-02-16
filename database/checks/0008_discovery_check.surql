-- 0008_discovery_check
-- track/stage fields removed by 0015_discovery_drop_track_stage.
INFO FOR TABLE discovery_feed_item;
INFO FOR TABLE discovery_notification;
SELECT count() AS discovery_feed_item_rows FROM discovery_feed_item;
SELECT count() AS discovery_notification_rows FROM discovery_notification;
