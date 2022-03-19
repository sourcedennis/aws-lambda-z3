# aws-lambda-z3

A *Tutorial* on setting up the [Z3 theorem prover](https://github.com/Z3Prover/z3), for use within a Rust program, within [AWS Lambda](https://aws.amazon.com/lambda/). The tutorial is mostly for myself, as I'll have forgotten this in two weeks. Hopefully it is useful for others too.

This Rust project is merely a *proof-of-concept*. It performs:

* Take a HTTP POST request with a value, in JSON (formatted as `{"val": 42}`).
* Invokes Z3 to find two non-zero positive integers `x` and `y` (which are less than `val`), which sum to `val`.
  * That is, `x > 0`, `y > 0`, `x < val`, `y < val`, `x + y = val`
* In the HTTP response, it returns those values, as JSON. (formatted as `{"x":6,"y":36}`).

Note that Z3 is a *dynamically linked* dependency of our program, which needs to be shipped along. To accomplish that, we pack everything into a Docker image, which we push to AWS.

The contents of both `src/main.rs` and `Dockerfile` are documented. The remainder of this README elaborates on getting the project into Lambda.

## Build the Docker image

Simply run:

```
docker build . --tag=aws-lambda-z3
```

Note that this pull [Z3 from GitHub](https://github.com/Z3Prover/z3) and builds it from scratch. This may take a while.

## Push the Docker image to AWS

* Create a Docker repository on [Amazon ECR](https://console.aws.amazon.com/ecr/home/).
* Push the image to the repository
  * Inside the repository (in AWS console), follow the instructions described in **View push commands**.

## Creating the Lambda Function

* Go to the [AWS Lambda Console](https://console.aws.amazon.com/lambda/home)
* Click **Create function**
* Select **Container image**
* Give your function a name (e.g., `aws-lambda-z3`)
* Select your pushed image in the **Container image URI** field

## Make the Function publicly accessible

* Click **+ Add trigger**
* Select for API: **Create an API**
* Select for API type: **REST API**
* Select for Security: **Open**
* Keep the default "Additional settings"
* Click **Add**

AWS offers Lambda Proxy Integration, which seemingly interferes with our program's ability to parse a request. For simplicity, disable it:

* Go to the [Amazon API Gateway Console](https://console.aws.amazon.com/apigateway/home)
* Select **Integration Request**
* **Uncheck** the box for **Use Lambda Proxy Integration**
* **Important!** Click `Actions > Deploy API` and redeploy to the default stage

Now the program should be publicly accessible through an URL - as listed in Lambda's trigger information. This URL looks like:
```
https://*.execute-api.*.amazonaws.com/default/aws-lambda-z3
```

## Testing

To test the Lambda function, make a POST request with (replace `[URL]`):
```
curl -X POST -d '{"val": 42}' "[URL]"
```
This should return:
```
{"x":6,"y":36}
```

Note that the [Amazon API Gateway Console](https://console.aws.amazon.com/apigateway/home) also offers utilities to perform HTTP requests.

Well done! Now you have Z3 running inside Lambda.

## Common Errors

While setting this up, I ran into several HTTP errors:

* `403` - Forbidden. This likely occurs when you've sent the request *to the wrong URL*.
* `500`/`502` - Internal Server Error. This happens when the program crashes. Possibly because the JSON string does not contain the `val` field. Alternatively, make sure *Lambda Proxy Integration* is disabled (and then redeploy); As it seemingly modifies the input JSON message.

## License

BSD-3 - See the `LICENSE` file
