#![allow(missing_docs, rustdoc::missing_crate_level_docs)]
// The `optimism` feature must be enabled to use this crate.
#![cfg(feature = "optimism")]

use clap::Parser;
use reth_node_builder::{engine_tree_config::TreeConfig, EngineNodeLauncher, Node};
use reth_optimism_cli::{chainspec::OpChainSpecParser, Cli};
use reth_optimism_node::{args::RollupArgs, OpNode};
use reth_provider::providers::BlockchainProvider;

use tracing as _;

#[global_allocator]
static ALLOC: reth_cli_util::allocator::Allocator = reth_cli_util::allocator::new_allocator();

fn main() {
    reth_cli_util::sigsegv_handler::install();

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(err) =
        Cli::<OpChainSpecParser, RollupArgs>::parse().run(|builder, rollup_args| async move {
            let engine_tree_config = TreeConfig::default()
                .with_persistence_threshold(rollup_args.persistence_threshold)
                .with_memory_block_buffer_target(rollup_args.memory_block_buffer_target);

            let op_node = OpNode::new(rollup_args.clone());
            let handle = builder
                .with_types_and_provider::<OpNode, BlockchainProvider<_>>()
                .with_components(op_node.components())
                .with_add_ons(op_node.add_ons())
                .launch_with_fn(|builder| {
                    let launcher = EngineNodeLauncher::new(
                        builder.task_executor().clone(),
                        builder.config().datadir(),
                        engine_tree_config,
                    );
                    builder.launch_with(launcher)
                })
                .await?;

            handle.node_exit_future.await
        })
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}