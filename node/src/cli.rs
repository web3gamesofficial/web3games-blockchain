// This file is part of Web3Games.

// Copyright (C) 2021 Web3Games https://web3games.org
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#[cfg(feature = "manual-seal")]
use structopt::clap::arg_enum;
use structopt::StructOpt;

#[cfg(feature = "manual-seal")]
arg_enum! {
	/// Available Sealing methods.
	#[derive(Debug, Copy, Clone, StructOpt)]
	pub enum Sealing {
		// Seal using rpc method.
		Manual,
		// Seal when transaction is executed.
		Instant,
	}
}

#[cfg(feature = "manual-seal")]
impl Default for Sealing {
	fn default() -> Sealing {
		Sealing::Manual
	}
}

#[allow(missing_docs)]
#[derive(Debug, StructOpt)]
pub struct RunCmd {
	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub base: sc_cli::RunCmd,

	#[cfg(feature = "manual-seal")]
	/// Choose sealing method.
	#[structopt(long = "sealing")]
	pub sealing: Sealing,

	#[structopt(long = "enable-dev-signer")]
	pub enable_dev_signer: bool,

	/// Maximum number of logs in a query.
	#[structopt(long, default_value = "10000")]
	pub max_past_logs: u32,

	/// The dynamic-fee pallet target gas price set by block author
	#[structopt(long, default_value = "1")]
	pub target_gas_price: u64,
}

#[derive(Debug, StructOpt)]
pub struct Cli {
	#[structopt(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[structopt(flatten)]
	pub run: RunCmd,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
	/// Key management cli utilities
	Key(sc_cli::KeySubcommand),
	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(sc_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// The custom benchmark subcommmand benchmarking runtime pallets.
	#[structopt(name = "benchmark", about = "Benchmark runtime pallets.")]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}
