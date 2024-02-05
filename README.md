# cronwrap-alert

Simple wrapper for cron jobs that posts alerts on error, written in Rust.

When the command fails, this wrapper will make a POST request to a configurable
URL with an `Authorization: Bearer <token>` header and the following JSON body:

```json
{
  "channel": "<channel>",
  "category": "<category>",
  "origin": "<origin>",
  "subject": "Scheduled execution failed: <name>",
  "contentType": "text/plain",
  "body": "<description>"
}
```

All fields marked with `<...>` are configurable through command-line options as
well as environment variables.

The description contains the start time, exit code, full path, all arguments,
and the standard error output of the failed command.
