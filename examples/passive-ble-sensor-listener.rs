// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! This example listens for BLE Service Advertisements using the `btleplug` crate, parses them and
//! then prints them to stdout.

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::error::Error;
use xiaomi_ble::parse_service_advertisement;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let manager = Manager::new().await?;

    // Get the first bluetooth adapter and connect to it.
    let adapters = manager.adapters().await.unwrap();
    let central = adapters.into_iter().next().unwrap();

    // Each adapter has an event stream, we fetch via events(),
    // simplifying the type, this will return what is essentially a
    // Future<Result<Stream<Item=CentralEvent>>>.
    let mut events = central.events().await?;

    // Start scanning for devices.
    central.start_scan(ScanFilter::default()).await?;

    // When getting a ServiceDataAdvertisement, print the senders's MAC address, name and the
    // contained sensor values.
    while let Some(event) = events.next().await {
        if let CentralEvent::ServiceDataAdvertisement { id, service_data } = &event {
            let mut service_advertisements = service_data
                .iter()
                .filter_map(|(uuid, data)| parse_service_advertisement(uuid, data).ok())
                .peekable();
            if service_advertisements.peek().is_some() {
                let peripheral = central.peripheral(id).await?;
                println!("MAC: {}", peripheral.address());
                let properties = peripheral.properties().await?;
                properties
                    .and_then(|p| p.local_name)
                    .inspect(|local_name| println!("Name: {}", local_name));

                for service_advertisement in service_advertisements {
                    for event in service_advertisement.iter_sensor_values() {
                        println!("Sensor Value: {:?}", event);
                    }
                }
                println!()
            }
        }
    }
    Ok(())
}
