# crate::launch
`crate::launch` launch a fullstack app based on `dioxus`.
It aims to provide it with same convention of `dioxus::launch`.

## Arguments
### Function parameter
- Same with `dioxus::launch`.

### Environment variables
| Variable      | Description                                     |
|---------------|-------------------------------------------------|
| SESSION_TABLE | Dynamo session table name for `session` feature |

## Features
### `lambda` feature
- `lambda` feature builds AWS Lambda binary instead of a real server.

### `session` feature
- `session` feature makes the app to use session based on DynamoDB.

## Usage
### Writing a main function

```rust
use dioxus::prelude::*;

fn main() {
    dioxus_aws::launch(app);
}

fn app() -> Element {
  rsx! {
     div { }
  }
}

```
