# RoWifi Cache
This crate handles the in-memory cache of a single cluster. It updates the data sequentially as the bot gets events from the Discord gateway. We use custom models to ensure we only store minimal data. All other fields are discarded. On top of that, we store some extra fields that are calculated from the events so that they don't have to be calculated every time a command is run.
