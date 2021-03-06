//
// Copyright (C) 2018 Kubos Corporation
//
// Licensed under the Apache License, Version 2.0 (the "License")
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

//! Kubos Service for interacting with [ISIS Antenna Systems](https://www.isispace.nl/product-category/products/antenna-systems/)
//!
//! # Configuration
//!
//! The service must be configured in `/home/system/etc/config.toml` with the following fields:
//!
//! - `[isis-ants-service.addr]`
//!
//!     - `ip` - Specifies the service's IP address
//!     - `port` - Specifies the port on which the service will be listening for UDP packets
//!
//! - `[isis-ants-service]`
//!
//!     - `bus` - Specifies the I2C bus the antenna system is connected to
//! 	- `primary` - Specifies the I2C address of the antenna system's primary microcontroller
//! 	- `secondary` - Specifies the I2C address of the secondary microcontroller. If no secondary contoller is present, this value should be `"0x00"`.
//! 	- `antennas` - Specifies the number of antennas present in the system. Expected value: 2 or 4.
//! 	- `wd_timeout` - Specifies the interval at which the AntS watchdog should be automatically kicked. To disable automatic kicking, this value should be `0`.
//!
//! For example:
//!
//! ```toml
//! [isis-ants-service.addr]
//! ip = "0.0.0.0"
//! port = 8006
//!
//! [isis-ants-service]
//! bus = "KI2C1"
//! primary = "0x31"
//! secondary = "0x32"
//! antennas = 4
//! wd_timeout = 10
//! ```
//!
//! # Starting the Service
//!
//! The service should be started automatically by its init script, but may also be started manually:
//!
//! ```shell
//! $ isis-ants-service
//! Kubos antenna systems service started
//! Listening on: 0.0.0.0:8006
//! ```
//!
//! # Available Fields
//!
//! ```json
//! query {
//!     ack
//!     armStatus
//!     config
//!     deploymentStatus {
//!         status,
//!         sysBurnActive,
//!         sysIgnoreDeploy,
//!         sysArmed,
//!         ant1NotDeployed,
//!         ant1StoppedTime,
//!         ant1Active,
//!         ant2NotDeployed,
//!         ant2StoppedTime,
//!         ant2Active,
//!         ant3NotDeployed,
//!         ant3StoppedTime,
//!         ant3Active,
//!         ant4NotDeployed,
//!         ant4StoppedTime,
//!         ant4Active
//!     }
//!     power {
//!         state,
//!         uptime
//!     }
//!     telemetry {
//!         nominal {
//!             rawTemp,
//!             uptime,
//!             sysBurnActive,
//!             sysIgnoreDeploy,
//!             sysArmed,
//!             ant1NotDeployed,
//!             ant1StoppedTime,
//!             ant1Active,
//!             ant2NotDeployed,
//!             ant2StoppedTime,
//!             ant2Active,
//!             ant3NotDeployed,
//!             ant3StoppedTime,
//!             ant3Active,
//!             ant4NotDeployed,
//!             ant4StoppedTime,
//!             ant4Active
//!         },
//!         debug {
//!             ant1ActivationCount,
//!             ant1ActivationTime,
//!             ant2ActivationCount,
//!             ant2ActivationTime,
//!             ant3ActivationCount,
//!             ant3ActivationTime,
//!             ant4ActivationCount,
//!             ant4ActivationTime,
//!         }
//!     }
//!     testResults{
//!         success,
//!         telemetryNominal{...},
//!         telemetryDebug{...}
//!     }
//!     errors
//! }
//!
//! mutation {
//!     arm(state: ArmState) {
//!         errors,
//!         success
//!     }
//!     configureHardware(config: ConfigureController) {
//!         errors,
//!         success,
//!         config
//!     }
//!     controlPower(state: PowerState) {
//!         errors,
//!         success,
//!         power
//!     }
//!     deploy(ant: DeployType, force: bool, time: i32) {
//!         errors,
//!         success
//!     }
//!     issueRawCommand(command: String, rx_len: i32) {
//!         errors,
//!         success,
//!         response
//!     }
//!     noop {
//!         errors,
//!         success
//!     }
//!     integration: testHardware(test: INTEGRATION) {
//!         ... on IntegrationTestRsults {
//!             errors,
//!             success,
//!             telemetryNominal{...},
//!             telemetryDebug{...}
//!         }
//!     }
//!     hardware: testHardware(test: HARDWARE) {
//!         ... on HardwareTestResults {
//!             errors,
//!             success,
//!             data
//!         }
//!     }
//! }
//! ```
//!

#![deny(missing_docs)]
#![recursion_limit = "256"]

#[cfg(test)]
#[macro_use]
extern crate double;
extern crate failure;
extern crate isis_ants_api;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate kubos_service;
#[cfg(test)]
#[macro_use]
extern crate serde_json;

use isis_ants_api::AntSResult;
use kubos_service::{Config, Service};
use model::Subsystem;
use schema::{MutationRoot, QueryRoot};

mod model;
mod objects;
mod schema;
#[cfg(test)]
mod tests;

fn main() -> AntSResult<()> {
    let config = Config::new("isis-ants-service");

    let bus = config
        .get("bus")
        .expect("No 'bus' value found in 'isis-ants-service' section of config");
    let bus = bus.as_str().unwrap();

    let primary = config
        .get("primary")
        .expect("No 'primary' value found in 'isis-ants-service' section of config");
    let primary = primary.as_str().unwrap();
    let primary: u8 = match primary.starts_with("0x") {
        true => u8::from_str_radix(&primary[2..], 16).unwrap(),
        false => u8::from_str_radix(primary, 16).unwrap(),
    };

    let secondary = config
        .get("secondary")
        .expect("No 'secondary' value found in 'isis-ants-service' section of config");
    let secondary = secondary.as_str().unwrap();
    let secondary: u8 = match secondary.starts_with("0x") {
        true => u8::from_str_radix(&secondary[2..], 16).unwrap(),
        false => u8::from_str_radix(secondary, 16).unwrap(),
    };

    let antennas = config
        .get("antennas")
        .expect("No 'antennas' value found in 'isis-ants-service' section of config");
    let antennas = antennas.as_integer().unwrap() as u8;

    let wd_timeout = config
        .get("wd_timeout")
        .expect("No 'wd_timeout' value found in 'isis-ants-service' section of config");
    let wd_timeout = wd_timeout.as_integer().unwrap() as u32;

    Service::new(
        config,
        Subsystem::new(bus, primary, secondary, antennas, wd_timeout)?,
        QueryRoot,
        MutationRoot,
    ).start();

    Ok(())
}
