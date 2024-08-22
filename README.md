# **EasyMq**
easy messagequeue

## **Quick Start**
### **Run**

**config.json example**
```json
{
  "log_root":"./",
  "rest_api_bind":"0.0.0.0:8080",
  "delay_duration":500
}
```
start easymq on terminal
```shell
easymq config.json
```

### Publish Message
**Http**
```shell
api_host=http://localhost:8080
curl $api_host/publish -X POST -H 'Content-Type:application/json' -d '{"topic":"test1","content":"good evening,sir"}'
```
### Read Latest Message
**Http**
```shell
api_host=http://localhost:8080
curl $api_host/read\?topic\=test1 -X GET
```