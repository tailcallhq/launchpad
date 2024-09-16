import * as pulumi from "@pulumi/pulumi"
import * as aws from "@pulumi/aws"

const config = new pulumi.Config()
const linkedConfig = config.require("linked_config")

let tailcallConfig = new pulumi.asset.StringAsset(
  `schema @server @upstream @link(type: Config, src: "${linkedConfig}") { query: Query }`,
)

const combinedArchive = new pulumi.asset.AssetArchive({
  "config.graphql": tailcallConfig,
  tailcall: new pulumi.asset.FileAsset("./tailcall"),
  bootstrap: new pulumi.asset.FileAsset("./bootstrap"),
})

const assumeRole = aws.iam.getPolicyDocument({
  statements: [
    {
      effect: "Allow",
      principals: [
        {
          type: "Service",
          identifiers: ["lambda.amazonaws.com"],
        },
      ],
      actions: ["sts:AssumeRole"],
    },
  ],
})

const iamRole = new aws.iam.Role("iam_role", {
  assumeRolePolicy: assumeRole.then((assumeRole) => assumeRole.json),
})

const lambdaFunction = new aws.lambda.Function("lambda", {
  role: iamRole.arn,
  code: combinedArchive,
  runtime: aws.lambda.Runtime.CustomAL2,
  handler: "start",
})

// TODO: add extra code to enable RestAPI to have a custom domain
const restApi = new aws.apigateway.RestApi("rest_api")

const apiResource = new aws.apigateway.Resource("api_resource", {
  restApi: restApi.id,
  parentId: restApi.rootResourceId,
  pathPart: "{proxy+}",
})

const apiMethod = new aws.apigateway.Method("api_method", {
  restApi: restApi.id,
  resourceId: apiResource.id,
  httpMethod: "ANY",
  authorization: "NONE",
  apiKeyRequired: false,
})

const apiIntegration = new aws.apigateway.Integration("api_integration", {
  restApi: restApi.id,
  resourceId: apiResource.id,
  httpMethod: apiMethod.httpMethod,
  integrationHttpMethod: "POST",
  type: "AWS_PROXY",
  uri: lambdaFunction.invokeArn,
})

const apiRootMethod = new aws.apigateway.Method("api_root_method", {
  restApi: restApi.id,
  resourceId: restApi.rootResourceId,
  httpMethod: "ANY",
  authorization: "NONE",
  apiKeyRequired: false,
})

const apiRootIntegration = new aws.apigateway.Integration("api_root_integration", {
  restApi: restApi.id,
  resourceId: restApi.rootResourceId,
  httpMethod: apiRootMethod.httpMethod,
  integrationHttpMethod: "POST",
  type: "AWS_PROXY",
  uri: lambdaFunction.invokeArn,
})

const apiDeployment = new aws.apigateway.Deployment(
  "api_deployment",
  {
    restApi: restApi.id,
    triggers: {
      redeployment: restApi.executionArn,
    },
  },
  {dependsOn: [apiMethod, apiRootMethod, apiIntegration, apiRootIntegration]},
)

const apiStage = new aws.apigateway.Stage("api_stage", {
  deployment: apiDeployment.id,
  restApi: restApi.id,
  stageName: "tailcall",
})

const apiMethodSettings = new aws.apigateway.MethodSettings("api_method_settings", {
  restApi: restApi.id,
  stageName: apiStage.stageName,
  methodPath: "*/*",
  settings: {},
})

const apiPermission = new aws.lambda.Permission("api_gateway_permission", {
  statementId: "AllowAPIGatewayInvoke",
  action: "lambda:InvokeFunction",
  function: lambdaFunction.name.apply((n) => n),
  principal: "apigateway.amazonaws.com",
  sourceArn: restApi.executionArn.apply((urn) => `${urn}/*/*`),
})

export const url = apiStage.invokeUrl
