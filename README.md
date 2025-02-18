# Solana Pump Fun Scanner & Rug Pull Checker

Solana Pump Fun Scanner is a Rust project that scans the Solana network for new pump fun launches, stores the token information in a PostgreSQL database, and iterates through the stored tokens to perform rug pull checks. The application leverages asynchronous programming with Tokio, interacts with Solana's network, and uses external APIs for comprehensive security analysis.

## Features

- **Pump Fun Scanner**: Monitors the Solana blockchain for new pump fun launch events.
- **Token Storage**: Automatically stores token details in a PostgreSQL database.
- **Rug Pull Checks**: Iterates through the database to evaluate tokens for potential rug pulls using multiple risk metrics.
- **Asynchronous Processing**: Built with Tokio for efficient async task handling.
- **Logging & Error Handling**: Provides informative logging throughout the process.

## Prerequisites

- **Rust**: Ensure you have Rust installed (v1.56+ is recommended).
- **Cargo**: Comes bundled with Rust.
- **PostgreSQL**: You need an instance of PostgreSQL to persist token data.
- **Solana CLI & RPC Endpoints**: (Optional) for interacting with the Solana network.

## Installation

### Installing Rust

If you don't have Rust installed, you can install it via `rustup`. Open your terminal and run:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, restart your terminal and verify by running:

```bash
rustc --version
cargo --version
```

### Clone the Repository

Clone the repository and navigate to the project directory:

```bash
git clone https://github.com/yourusername/solana-pump-fun-scanner.git
cd solana-pump-fun-scanner
```

### Configure Environment Variables

Create a `.env` file in the root of the project with the following environment variables:

```ini
DATABASE_URL=postgres://username:password@localhost:5432/database_name
GRPC_ENDPOINT=your_grpc_endpoint
```

Adjust the values to match your local configuration and credentials.

### Build the Project

Compile the project with Cargo:

```bash
cargo build --release
```

## Running the Project

Run the project using Cargo:

```bash
cargo run --release
```

The application will:
- Connect to your PostgreSQL instance.
- Begin scanning the Solana network for pump fun launches.
- Store token information in the database.
- Continuously check stored tokens for potential rug pulls.

## Project Structure

- **src/main.rs**: Main entry point. Sets up asynchronous tasks such as scanning the network, processing tokens, and connecting to gRPC streams.
- **src/managers/db_manager.rs**: Implements database operations (e.g., inserting token info, deleting tokens based on rug pull checks).
- **src/utils/rug_check.rs**: Contains logic for making rug pull assessments by integrating external API data.
- **src/models/token.rs**: Defines the data structures for token information.

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository.
2. Create a new branch for your feature or fix.
3. Submit a pull request with a detailed description of your changes.

Please follow the project's coding guidelines and include tests where applicable.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For questions or feedback, please open an issue in the GitHub repository or contact the maintainer at [email@example.com](mailto:email@example.com).
