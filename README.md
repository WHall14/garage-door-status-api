# garage-door-status-api
API built on AWS API Gateway with Lambda and DynamoDB Backend


## OpenAPI Codegen
Install the OpenAPI Generator, I used `npm` for this 
```bash
  npm install -g @openapitools/openapi-generator-cli
```

### Status - Update
Generate the models from the top level dir
```bash
  openapi-generator-cli generate -i openapi.yaml -g rust -o garage-status/src/generated --global-property models,modelDocs=false
```

## Generate ZIP
```bash
cargo lambda build --release --output-format zip
```
