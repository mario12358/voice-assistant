use cpal::traits::HostTrait;

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDevice {
    pub name: String,
    pub is_default: bool,
}

/// Źródło listy mikrofonów — w aplikacji CoreAudio przez cpal, w testach fałszywka.
pub trait AudioHost {
    fn input_devices(&self) -> Result<Vec<InputDevice>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceChoice {
    pub name: String,
    /// `true`, gdy mikrofonu z konfiguracji nie ma i użyto domyślnego.
    pub fell_back: bool,
}

/// Wybiera mikrofon: ten z konfiguracji, a gdy go nie ma — domyślny systemu.
pub fn choose_device(devices: &[InputDevice], preferred: Option<&str>) -> Result<DeviceChoice> {
    if let Some(name) = preferred
        && devices.iter().any(|device| device.name == name)
    {
        return Ok(DeviceChoice {
            name: name.to_owned(),
            fell_back: false,
        });
    }
    let fallback = devices
        .iter()
        .find(|device| device.is_default)
        .or_else(|| devices.first())
        .ok_or(Error::NoInputDevice)?;
    if let Some(missing) = preferred {
        tracing::warn!(
            missing,
            used = fallback.name,
            "wybrany mikrofon niedostępny, używam domyślnego"
        );
    }
    Ok(DeviceChoice {
        name: fallback.name.clone(),
        fell_back: preferred.is_some(),
    })
}

pub struct CpalHost {
    host: cpal::Host,
}

impl CpalHost {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }
}

impl Default for CpalHost {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioHost for CpalHost {
    fn input_devices(&self) -> Result<Vec<InputDevice>> {
        let default_name = self
            .host
            .default_input_device()
            .map(|device| device.to_string());
        let devices = self.host.input_devices().map_err(backend)?;
        Ok(devices
            .map(|device| {
                let name = device.to_string();
                InputDevice {
                    is_default: default_name.as_deref() == Some(name.as_str()),
                    name,
                }
            })
            .collect())
    }
}

fn backend(error: impl std::fmt::Display) -> Error {
    Error::Backend(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(name: &str, is_default: bool) -> InputDevice {
        InputDevice {
            name: name.into(),
            is_default,
        }
    }

    fn mac_devices() -> Vec<InputDevice> {
        vec![
            device("MacBook Pro Microphone", true),
            device("USB Audio Device", false),
            device("AirPods Pro", false),
        ]
    }

    #[test]
    fn configured_microphone_is_chosen_when_present() {
        let choice = choose_device(&mac_devices(), Some("USB Audio Device")).unwrap();

        assert_eq!(
            choice,
            DeviceChoice {
                name: "USB Audio Device".into(),
                fell_back: false
            }
        );
    }

    #[test]
    fn no_configured_microphone_uses_system_default() {
        let choice = choose_device(&mac_devices(), None).unwrap();

        assert_eq!(choice.name, "MacBook Pro Microphone");
        assert!(!choice.fell_back);
    }

    // specky: crit 01M4EKHCZM97NBJHT663SGREHW
    #[test]
    fn missing_configured_microphone_falls_back_to_system_default() {
        let choice = choose_device(&mac_devices(), Some("Odłączony mikrofon")).unwrap();

        assert_eq!(choice.name, "MacBook Pro Microphone");
        assert!(choice.fell_back);
    }

    #[test]
    fn without_marked_default_first_device_is_used() {
        let devices = vec![
            device("USB Audio Device", false),
            device("AirPods Pro", false),
        ];

        let choice = choose_device(&devices, Some("Odłączony mikrofon")).unwrap();

        assert_eq!(choice.name, "USB Audio Device");
    }

    #[test]
    fn no_devices_is_a_readable_error() {
        let error = choose_device(&[], Some("USB Audio Device")).unwrap_err();

        assert!(matches!(error, Error::NoInputDevice));
        assert!(
            error
                .to_string()
                .contains("nie znaleziono żadnego mikrofonu")
        );
    }

    #[test]
    #[ignore = "wymaga mikrofonu w systemie"]
    fn cpal_host_lists_exactly_one_default_microphone() {
        let devices = CpalHost::new().input_devices().unwrap();

        assert!(!devices.is_empty());
        assert_eq!(devices.iter().filter(|device| device.is_default).count(), 1);
    }
}
