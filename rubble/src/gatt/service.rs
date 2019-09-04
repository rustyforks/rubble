use {
    crate::{att::AttUuid, gatt::characteristic::{Characteristic, Appearance}},
};

/// Trait for specifying services exposed via GATT.
///
/// An implementation of `ServiceSpec` is essentially a machine-readable transcript of one of the
/// service specifications available [here][service-specs]. It can be *instantiated* by the
/// `rubble-codegen` crate, resulting in a set of attributes and handlers.
///
/// [service-specs]: https://www.bluetooth.com/specifications/gatt/
pub trait ServiceSpec {
    /// Whether this service can be used as a primary or secondary service, or either.
    const ALLOWED_TYPE: AllowedType;

    /// UUID identifying the service.
    ///
    /// If this is a 16-bit UUID, it must be assigned by the Bluetooth SIG. If it's a 128-bit UUID,
    /// it can be a randomly generated UUIDv4 and no SIG assignment is needed. This makes 128-bit
    /// UUIDs useful for custom services which have not been standardized by the SIG.
    const UUID: AttUuid;

    /// Whether only one instance of this service is allowed on a device.
    const SINGLETON: bool;

    /// Iterator over characteristic specifications.
    type Characteristics: Iterator<Item = AttUuid>;
}

/// The type of a service (primary or secondary).
#[derive(Copy, Clone, Debug)]
pub enum AllowedType {
    /// Primary service providing the main functionality of the device.
    ///
    /// Primary services can be discovered by a connected device using the *Primary Service
    /// Discovery* procedure.
    Primary,

    /// Secondary service included by another service.
    ///
    /// Secondary services are not discoverable on their own, but are instead part of the
    /// *includes* list of some other service.
    Secondary,

    /// Service can be a primary or secondary service, the service specification imposes no
    /// requirement.
    ///
    /// The actual service type used can depend on the implemented *Profile*.
    Any,
}

pub struct GapService {}

impl GapService {
    pub fn new(device_name: &str, appearance: Appearance) {}
}
