# SWHR [Simple WebHook Runner]

run scripts on HTTP request

## Usage

CLI Arguments:

`--config` - custom config name/location

`--listen` - custom listen URL

```yaml
services:
  - path: "/service1"
    method: "POST"
    dir: "/home/user/test_d1"
    script: "/home/user/script_1"
    api_key: "ANY_UTF8_STRING" -- optional

  - path: "/service2"
    method: "GET"
    dir: "/home/user/test_d2"
    script: "/home/user/script_2"
```

Webhook body is passed in `WEBHOOK_BODY` environment variable; **only UTF-8 valid Strings are supported**

```bash
echo $WEBHOOK_BODY # prints the body
```
