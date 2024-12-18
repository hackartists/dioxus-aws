# Dioxus AWS

- [Development](#org1151159)
  - [Creating a project](#org4a4e0bb)
  - [Re-Writing `main` function](#orgcd58878)
    - [Installing dependancy](#org850414d)
    - [`main` function](#org5e61e6c)
  - [Running and testing the project](#orgbb6d4af)
- [Deployment](#orgada4ed9)
  - [Setup AWS CDK](#org0f7dc9c)
  - [Writing CDK Stack](#org98abf74)
  - [Build and deploy application](#orgb164413)
    - [Building a binary for AWS Lambda](#org216cb45)
    - [Deploy AWS CDK](#org45f2978)

`dioxus-aws` crate provides a `launch` function to make `dioxus` run on AWS Serverless Stack (AWS Lambda, CloudFront and S3).


<a id="org1151159"></a>

# Development


<a id="org4a4e0bb"></a>

## Creating a project

-   Use `dx` command.
    -   Currently, , which is stable version of dioxus, is supported.

```shell
cargo install dioxus-cli --version ^0.5
dx new --subtemplate Fullstack project-name
```


<a id="orgcd58878"></a>

## Re-Writing `main` function


<a id="org850414d"></a>

### Installing dependancy

-   Add `dioxus-aws`.

```shell
cargo add dioxus-aws
```


<a id="org5e61e6c"></a>

### `main` function

-   Change `launch(App)` to `dioxus_aws::launch(App)`.

```rust
dioxus_aws::launch(App); // launch(App);
```


<a id="orgbb6d4af"></a>

## Running and testing the project

-   It is same with usage of `dioxus-cli`.

```shell
cargo add dioxus-aws
dx serve --platform web
```


<a id="orgada4ed9"></a>

# Deployment

It uses AWS Lambda, S3 and CloudFront to deploy dixous application.


<a id="org0f7dc9c"></a>

## Setup AWS CDK

-   Setup AWS CDK to deploy application.
    -   For more information about AWS CDK, refer to [AWS CDK Guide](https://docs.aws.amazon.com/cdk/v2/guide/getting_started.html)
    -   For basic tutorial of AWS CDK, refer to [the tutorial](https://docs.aws.amazon.com/cdk/v2/guide/hello_world.html)

```shell
npm install -g aws-cdk
mkdir -p fixtures/aws-cdk
cd fixtures/aws-cdk
cdk init app --language=typescript
```


<a id="org98abf74"></a>

## Writing CDK Stack

-   `Route53` sets up a domain to CloudFront.
-   `CloudFront` distributes requests to API Gateway or S3.
-   `API Gateway` is bound to AWS Lambda.
-   `S3` stores assets like js, html, images and so on.

```typescript
import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import * as cloudfront from "aws-cdk-lib/aws-cloudfront";
import * as origins from "aws-cdk-lib/aws-cloudfront-origins";
import * as s3 from "aws-cdk-lib/aws-s3";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as targets from "aws-cdk-lib/aws-route53-targets";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as apigateway from "aws-cdk-lib/aws-apigateway";

export class AwsCdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const domain = process.env.DOMAIN || "";
    const acmId = process.env.ACM_ID || "";
    const hostedZoneId = process.env.HOSTED_ZONE_ID || "";
    const projectRoot = process.env.PROJECT_ROOT || "";

    const assetsBucket = new s3.Bucket(this, "Bucket", {
      bucketName: domain,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
    });

    const certificate = acm.Certificate.fromCertificateArn(
      this,
      "Certificate",
      acmId,
    );

    const func = new lambda.Function(this, "Function", {
      runtime: lambda.Runtime.PROVIDED_AL2023,
      code: lambda.Code.fromAsset(projectRoot + "/dist"),
      handler: "bootstrap",
      environment: {
        NO_COLOR: "true",
        ASSETS_PATH: "./",
      },
      memorySize: 128,
    });

    const api = new apigateway.LambdaRestApi(this, "Api", {
      handler: func,
      proxy: true,
    });

    const s3Origin = new origins.S3Origin(assetsBucket);
    const apiOrigin = new origins.RestApiOrigin(api);

    const cf = new cloudfront.Distribution(this, "Distribution", {
      defaultBehavior: {
        origin: apiOrigin,
        cachePolicy: cloudfront.CachePolicy.CACHING_DISABLED,
        allowedMethods: cloudfront.AllowedMethods.ALLOW_ALL,
        cachedMethods: cloudfront.CachedMethods.CACHE_GET_HEAD_OPTIONS,
        originRequestPolicy:
          cloudfront.OriginRequestPolicy.ALL_VIEWER_EXCEPT_HOST_HEADER,
      },
      additionalBehaviors: {
        "/assets/*": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.js": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.css": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.html": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.ico": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/*.svg": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/icons/*": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
        "/images/*": {
          origin: s3Origin,
          cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
        },
      },
      domainNames: [domain],
      certificate,
    });

    const zone = route53.HostedZone.fromHostedZoneAttributes(
      this,
      "zone-attribute",
      {
        zoneName: domain,
        hostedZoneId,
      },
    );

    new route53.ARecord(this, "IpV4Record", {
      zone,
      target: route53.RecordTarget.fromAlias(new targets.CloudFrontTarget(cf)),
    });

    new route53.AaaaRecord(this, "IpV6Record", {
      zone,
      target: route53.RecordTarget.fromAlias(new targets.CloudFrontTarget(cf)),
    });
  }
}
```


<a id="orgb164413"></a>

## Build and deploy application


<a id="org216cb45"></a>

### Building a binary for AWS Lambda

-   Note that `server-feature` is set to `lambda` instead of `server`.
-   Then, rename binary to `bootstrap`.
    -   Usually, `SERVICE` might be `basename $(git rev-parse --show-toplevel)`.

```shell
export SERVICE=$(cargo tree | head -n 1 | awk '{print $1}')
dx build --release --platform fullstack --server-feature lambda
mv dist/$SERVICE dist/bootstrap
```


<a id="org45f2978"></a>

### Deploy AWS CDK

-   Let you remember environments in CDK Stack.
    -   `DOMAIN` is FQDN including subdomain.
    -   `ACM_ID` must be placed in `us-east-1` for CloudFront.
    -   `HOSTED_ZONE_ID` is a zone ID of Route53.
    -   `PROJECT_ROOT` is a path of project root.

```shell
export DOMAIN="dioxus.example.com"
export ACM_ID="arn:aws:acm:us-east-1:---"
export HOSTED_ZONE_ID="Z--"
export PROJECT_ROOT=$(pwd)
export AWS_PROFILE=default

cd fixtures/aws-cdk
npm run build
cdk synth
cdk bootstrap --profile $AWS_PROFILE

# AWS Stack deployment
cdk deploy --require-approval never --profile $AWS_PROFILE

# S3 Assets sync
aws s3 sync $PROJECT_ROOT/dist/public s3://$DOMAIN --delete --profile $AWS_PROFILE
```
