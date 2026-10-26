use super::super::{
    Fault, RequestToken,
    budget::{Budget, Permit},
};
use crate::{RawJson, protocol::v1 as sdk};
use std::{collections::BTreeMap, sync::Arc};

pub(in super::super) struct Session {
    pub generation: Option<RequestToken>,
    pub setup: Option<RequestToken>,
    pub ready: bool,
    pub prompt: Option<RequestToken>,
    pub closing: Option<RequestToken>,
    pub observed: bool,
    pub observed_parent: Option<(String, Permit)>,
    pub observed_cancel: bool,
    pub cancelled: bool,
    pub modes: Option<sdk::SessionModeState>,
    config: Vec<sdk::SessionConfigOption>,
    config_v2: Vec<crate::protocol::v2::SessionConfigOption>,
    _identity: Permit,
    _modes: Option<Permit>,
    _config: Option<Permit>,
}

impl Session {
    pub fn new(id: &str, budget: &Arc<Budget>) -> Result<Self, Fault> {
        Ok(Self {
            generation: None,
            setup: None,
            ready: false,
            prompt: None,
            closing: None,
            observed: false,
            observed_parent: None,
            observed_cancel: false,
            cancelled: false,
            modes: None,
            config: Vec::new(),
            config_v2: Vec::new(),
            _identity: budget.reserve(id.len().saturating_mul(3).saturating_add(512))?,
            _modes: None,
            _config: None,
        })
    }

    pub fn replace(&mut self, raw: &RawJson, budget: &Arc<Budget>) -> Result<(), Fault> {
        if let Some(modes) = raw.member("modes")?
            && modes.text() != "null"
        {
            let credit = budget.reserve(retained_size(&modes))?;
            self.modes = Some(crate::codec::deserialize(&modes)?);
            self._modes = Some(credit);
        }
        if let Some(config) = raw.member("configOptions")?
            && config.text() != "null"
        {
            let credit = budget.reserve(retained_size(&config))?;
            self.config = crate::codec::deserialize(&config)?;
            self._config = Some(credit);
        }
        Ok(())
    }

    pub fn mode(&self, raw: &RawJson) -> Result<(), Fault> {
        let request: sdk::SetSessionModeRequest = crate::codec::deserialize(raw)?;
        let modes = self
            .modes
            .as_ref()
            .ok_or(Fault::UnsupportedCapability("SessionModes"))?;
        if !modes
            .available_modes
            .iter()
            .any(|mode| mode.id == request.mode_id)
        {
            return Err(Fault::InvalidArgument("ModeId"));
        }
        Ok(())
    }
    pub fn replace_version(
        &mut self,
        raw: &RawJson,
        budget: &Arc<Budget>,
        version: crate::schema::Version,
    ) -> Result<(), Fault> {
        if version == crate::schema::Version::V1 {
            return self.replace(raw, budget);
        }
        if let Some(config) = raw.member("configOptions")?
            && config.text() != "null"
        {
            let credit = budget.reserve(retained_size(&config))?;
            self.config_v2 = crate::codec::deserialize(&config)?;
            self._config = Some(credit);
        }
        Ok(())
    }
    pub fn config_v2(&self, raw: &RawJson) -> Result<(), Fault> {
        use crate::protocol::v2::{
            SessionConfigKind as Kind, SessionConfigOptionValue as Value,
            SessionConfigSelectOptions as Options,
        };
        let request: crate::protocol::v2::SetSessionConfigOptionRequest =
            crate::codec::deserialize(raw)?;
        let option = self
            .config_v2
            .iter()
            .find(|option| option.config_id == request.config_id)
            .ok_or(Fault::UnsupportedCapability("SessionConfigOption"))?;
        let valid = match (&option.kind, &request.value) {
            (Kind::Boolean(_), Value::Boolean { .. }) => true,
            (Kind::Select(select), Value::Id { value }) => match &select.options {
                Options::Ungrouped(options) => options.iter().any(|option| &option.value == value),
                Options::Grouped(groups) => groups
                    .iter()
                    .any(|group| group.options.iter().any(|option| &option.value == value)),
                _ => false,
            },
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(Fault::InvalidArgument("ConfigValue"))
        }
    }

    pub fn config(&self, raw: &RawJson) -> Result<(), Fault> {
        use sdk::{
            SessionConfigKind as Kind, SessionConfigOptionValue as Value,
            SessionConfigSelectOptions as Options,
        };
        let request: sdk::SetSessionConfigOptionRequest = crate::codec::deserialize(raw)?;
        let option = self
            .config
            .iter()
            .find(|option| option.id == request.config_id)
            .ok_or(Fault::UnsupportedCapability("SessionConfigOption"))?;
        let valid = match (&option.kind, &request.value) {
            (Kind::Boolean(_), Value::Boolean { .. }) => true,
            (Kind::Select(select), Value::ValueId { value }) => match &select.options {
                Options::Ungrouped(options) => options.iter().any(|option| &option.value == value),
                Options::Grouped(groups) => groups
                    .iter()
                    .any(|group| group.options.iter().any(|option| &option.value == value)),
                _ => false,
            },
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(Fault::InvalidArgument("ConfigValue"))
        }
    }
}

pub(super) fn id(raw: &RawJson) -> Result<String, Fault> {
    let raw = raw.member("sessionId")?.ok_or(Fault::InvalidMessage)?;
    crate::codec::deserialize(&raw).map_err(Into::into)
}

pub(super) fn retained_size(raw: &RawJson) -> usize {
    // Account for decoded SDK strings, collection nodes and tiny JSON values.
    raw.text().len().saturating_mul(16).saturating_add(512)
}

pub(super) type Sessions = BTreeMap<String, Session>;
