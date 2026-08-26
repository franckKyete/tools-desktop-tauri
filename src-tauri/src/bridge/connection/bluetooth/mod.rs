mod adapter1;
mod profile;
mod profile_manager1;
mod socket;

pub use socket::BluetoothSocket;

use std::{collections::HashMap, error::Error, future::Future, sync::Arc};

use log::info;
use tokio::runtime::Handle;
use zbus::{
    Connection,
    zvariant::{ObjectPath, Value},
};

use adapter1::Adapter1Proxy;

use profile::Profile;
use profile_manager1::ProfileManager1Proxy;

use crate::bridge::connection::Socket;

// #[derive(Clone, Copy)]
// pub enum BluetoothDeviceType {
//     Phone,
//     Laptop,
//     Other,
// }
//
// #[derive(Clone, Debug)]
// pub struct BluetoothDevice {
//     name: String,
//     address: String,
//     // r#type: BluetoothDeviceType,
//     // connected: bool,
// }

// impl BluetoothDevice {
//     pub fn from_props(props: &HashMap<String, OwnedValue>) -> Self {
//         Self {
//             name: props.get("Name").unwrap().to_string(),
//             address: props.get("Address").unwrap().to_string(),
//         }
//     }
// }

#[allow(unused)]
pub struct Bluetooth {
    bus: Connection,
    pub address: String,
}

const UUID: &str = "00001101-0000-1000-8000-00805f9b34fb";

impl Bluetooth {
    pub async fn available() -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }

    // Initialize the dbus connection, enable Bluetooth if not already enabled and start the
    // SerialPort profile
    pub async fn init<F, Fut>(callback: F) -> Result<Self, Box<dyn Error>>
    where
        F: Fn(Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let bus = Connection::system().await?;
        bus.request_name("com.tools").await?;
        let adapter = Adapter1Proxy::new(&bus).await?;

        let callback = Arc::new(move |socket| Box::pin(callback(socket)));
        let callback = {
            // let callback = callback.clone();
            move |sock| {
                let callback = callback.clone();
                async move {
                    let socket = BluetoothSocket::new(sock);
                    callback(Socket::BT(socket)).await;
                }
            }
        };

        if adapter.powered().await? {
            info!("Bluetooth already turned on");
        } else {
            adapter.set_powered(true).await?;
            info!("Bluetooth turned on");
        }

        let rt_handle = Handle::current();
        //
        let profile = Profile::new(rt_handle.clone(), callback);
        // let profile = Profile::new(rt_handle.clone(), callback);

        bus.object_server()
            .at("/org/bluez/Profile", profile)
            .await?;
        println!("Profile served");

        let profile_manager = ProfileManager1Proxy::new(&bus).await?;

        let mut options: HashMap<&str, Value> = HashMap::new();
        options.insert("Name", Value::from("SerialPortService"));
        options.insert("Role", Value::from("server"));
        options.insert("AutoConnect", Value::from(true));

        profile_manager
            .register_profile(
                &ObjectPath::from_static_str("/org/bluez/Profile")?,
                UUID,
                options,
            )
            .await?;
        println!("Profile registered");

        let address = adapter.address().await?;
        Result::Ok(Self { bus, address })
    }
    // pub async fn devices(&self) -> Result<Vec<BluetoothDevice>, Box<dyn Error>> {
    //     let object_manager = ObjectManagerProxy::new(&self.bus, "org.bluez", "/").await?;
    //     let managed_objects = object_manager.get_managed_objects().await?;
    //
    //     let mut devices: Vec<BluetoothDevice> = vec![];
    //
    //     for (_, interfaces) in managed_objects.iter() {
    //         if let Some(device_props) = interfaces.get("org.bluez.Device1") {
    //             // Check if the device is connected
    //             devices.push(BluetoothDevice::from_props(device_props));
    //         }
    //     }
    //
    //     Result::Ok(devices)
    // }
    // pub async fn scan(&self) -> Vec<BluetoothDevice> {
    //     todo!()
    // }
    // pub async fn connect(&self, device: BluetoothDevice) -> BluetoothDevice {
    //     todo!()
    // }
}
