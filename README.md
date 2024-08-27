# Exa SDK

A Rust SDK for interacting with [Exa](https://exa.ai), providing easy access to advanced search and content retrieval capabilities.

## Abstract

The Exa SDK is a Rust library that simplifies interaction with the Exa API. It offers a set of high-level functions for performing searches, finding similar content, and retrieving detailed information about web pages. This SDK handles authentication, request construction, and response parsing, allowing developers to focus on utilizing Exa's powerful features in their applications.

## Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
exa-sdk = { git = "https://github.com/fdionisi/exa-sdk" }
```

## Usage

Here's a basic example of how to use the Exa SDK:

```rust
use exa_sdk::{Exa, SearchRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Exa client
    let exa = Exa::builder()
        .api_key("your_api_key_here".to_string())
        .build()?;

    // Create a search request
    let request = SearchRequest {
        query: "Rust programming".to_string(),
        ..Default::default()
    };

    // Perform the search
    let response = exa.search(request).await?;

    // Process the results
    for result in response.results {
        println!("Title: {}", result.title);
        println!("URL: {}", result.url);
        println!("Score: {}", result.score);
        println!("---");
    }

    Ok(())
}
```

For more detailed usage examples, including how to use advanced features like finding similar content or retrieving detailed page information, please refer to the documentation of each module.

## License

exa-sdk is distributed under the terms of the MIT license.

See [LICENSE](LICENSE) for details.
