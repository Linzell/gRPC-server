impl serde::Serialize for Language {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::English => "LANGUAGE_ENGLISH",
            Self::Spanish => "LANGUAGE_SPANISH",
            Self::French => "LANGUAGE_FRENCH",
            Self::German => "LANGUAGE_GERMAN",
            Self::Italian => "LANGUAGE_ITALIAN",
            Self::Japanese => "LANGUAGE_JAPANESE",
            Self::Korean => "LANGUAGE_KOREAN",
            Self::Chinese => "LANGUAGE_CHINESE",
            Self::Russian => "LANGUAGE_RUSSIAN",
            Self::Arabic => "LANGUAGE_ARABIC",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Language {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "LANGUAGE_ENGLISH",
            "LANGUAGE_SPANISH",
            "LANGUAGE_FRENCH",
            "LANGUAGE_GERMAN",
            "LANGUAGE_ITALIAN",
            "LANGUAGE_JAPANESE",
            "LANGUAGE_KOREAN",
            "LANGUAGE_CHINESE",
            "LANGUAGE_RUSSIAN",
            "LANGUAGE_ARABIC",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Language;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "LANGUAGE_ENGLISH" => Ok(Language::English),
                    "LANGUAGE_SPANISH" => Ok(Language::Spanish),
                    "LANGUAGE_FRENCH" => Ok(Language::French),
                    "LANGUAGE_GERMAN" => Ok(Language::German),
                    "LANGUAGE_ITALIAN" => Ok(Language::Italian),
                    "LANGUAGE_JAPANESE" => Ok(Language::Japanese),
                    "LANGUAGE_KOREAN" => Ok(Language::Korean),
                    "LANGUAGE_CHINESE" => Ok(Language::Chinese),
                    "LANGUAGE_RUSSIAN" => Ok(Language::Russian),
                    "LANGUAGE_ARABIC" => Ok(Language::Arabic),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Notifications {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.email {
            len += 1;
        }
        if self.push {
            len += 1;
        }
        if self.sms {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.Notifications", len)?;
        if self.email {
            struct_ser.serialize_field("email", &self.email)?;
        }
        if self.push {
            struct_ser.serialize_field("push", &self.push)?;
        }
        if self.sms {
            struct_ser.serialize_field("sms", &self.sms)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Notifications {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "email",
            "push",
            "sms",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Email,
            Push,
            Sms,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "email" => Ok(GeneratedField::Email),
                            "push" => Ok(GeneratedField::Push),
                            "sms" => Ok(GeneratedField::Sms),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Notifications;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.Notifications")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Notifications, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut email__ = None;
                let mut push__ = None;
                let mut sms__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Email => {
                            if email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("email"));
                            }
                            email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Push => {
                            if push__.is_some() {
                                return Err(serde::de::Error::duplicate_field("push"));
                            }
                            push__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Sms => {
                            if sms__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sms"));
                            }
                            sms__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Notifications {
                    email: email__.unwrap_or_default(),
                    push: push__.unwrap_or_default(),
                    sms: sms__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.Notifications", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Privacy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.data_collection {
            len += 1;
        }
        if self.location {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.Privacy", len)?;
        if self.data_collection {
            struct_ser.serialize_field("dataCollection", &self.data_collection)?;
        }
        if self.location {
            struct_ser.serialize_field("location", &self.location)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Privacy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data_collection",
            "dataCollection",
            "location",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DataCollection,
            Location,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "dataCollection" | "data_collection" => Ok(GeneratedField::DataCollection),
                            "location" => Ok(GeneratedField::Location),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Privacy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.Privacy")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Privacy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data_collection__ = None;
                let mut location__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DataCollection => {
                            if data_collection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataCollection"));
                            }
                            data_collection__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Privacy {
                    data_collection: data_collection__.unwrap_or_default(),
                    location: location__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.Privacy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Security {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.two_factor {
            len += 1;
        }
        if !self.qr_code.is_empty() {
            len += 1;
        }
        if self.magic_link {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.Security", len)?;
        if self.two_factor {
            struct_ser.serialize_field("twoFactor", &self.two_factor)?;
        }
        if !self.qr_code.is_empty() {
            struct_ser.serialize_field("qrCode", &self.qr_code)?;
        }
        if self.magic_link {
            struct_ser.serialize_field("magicLink", &self.magic_link)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Security {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "two_factor",
            "twoFactor",
            "qr_code",
            "qrCode",
            "magic_link",
            "magicLink",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TwoFactor,
            QrCode,
            MagicLink,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "twoFactor" | "two_factor" => Ok(GeneratedField::TwoFactor),
                            "qrCode" | "qr_code" => Ok(GeneratedField::QrCode),
                            "magicLink" | "magic_link" => Ok(GeneratedField::MagicLink),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Security;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.Security")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Security, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut two_factor__ = None;
                let mut qr_code__ = None;
                let mut magic_link__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TwoFactor => {
                            if two_factor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("twoFactor"));
                            }
                            two_factor__ = Some(map_.next_value()?);
                        }
                        GeneratedField::QrCode => {
                            if qr_code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("qrCode"));
                            }
                            qr_code__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MagicLink => {
                            if magic_link__.is_some() {
                                return Err(serde::de::Error::duplicate_field("magicLink"));
                            }
                            magic_link__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Security {
                    two_factor: two_factor__.unwrap_or_default(),
                    qr_code: qr_code__.unwrap_or_default(),
                    magic_link: magic_link__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.Security", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Settings {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.language.is_some() {
            len += 1;
        }
        if self.theme.is_some() {
            len += 1;
        }
        if self.notifications.is_some() {
            len += 1;
        }
        if self.privacy.is_some() {
            len += 1;
        }
        if self.security.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.Settings", len)?;
        if let Some(v) = self.language.as_ref() {
            let v = Language::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("language", &v)?;
        }
        if let Some(v) = self.theme.as_ref() {
            let v = Theme::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("theme", &v)?;
        }
        if let Some(v) = self.notifications.as_ref() {
            struct_ser.serialize_field("notifications", v)?;
        }
        if let Some(v) = self.privacy.as_ref() {
            struct_ser.serialize_field("privacy", v)?;
        }
        if let Some(v) = self.security.as_ref() {
            struct_ser.serialize_field("security", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Settings {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "language",
            "theme",
            "notifications",
            "privacy",
            "security",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Language,
            Theme,
            Notifications,
            Privacy,
            Security,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "language" => Ok(GeneratedField::Language),
                            "theme" => Ok(GeneratedField::Theme),
                            "notifications" => Ok(GeneratedField::Notifications),
                            "privacy" => Ok(GeneratedField::Privacy),
                            "security" => Ok(GeneratedField::Security),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Settings;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.Settings")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Settings, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut language__ = None;
                let mut theme__ = None;
                let mut notifications__ = None;
                let mut privacy__ = None;
                let mut security__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Language => {
                            if language__.is_some() {
                                return Err(serde::de::Error::duplicate_field("language"));
                            }
                            language__ = map_.next_value::<::std::option::Option<Language>>()?.map(|x| x as i32);
                        }
                        GeneratedField::Theme => {
                            if theme__.is_some() {
                                return Err(serde::de::Error::duplicate_field("theme"));
                            }
                            theme__ = map_.next_value::<::std::option::Option<Theme>>()?.map(|x| x as i32);
                        }
                        GeneratedField::Notifications => {
                            if notifications__.is_some() {
                                return Err(serde::de::Error::duplicate_field("notifications"));
                            }
                            notifications__ = map_.next_value()?;
                        }
                        GeneratedField::Privacy => {
                            if privacy__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privacy"));
                            }
                            privacy__ = map_.next_value()?;
                        }
                        GeneratedField::Security => {
                            if security__.is_some() {
                                return Err(serde::de::Error::duplicate_field("security"));
                            }
                            security__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Settings {
                    language: language__,
                    theme: theme__,
                    notifications: notifications__,
                    privacy: privacy__,
                    security: security__,
                })
            }
        }
        deserializer.deserialize_struct("client.v1.Settings", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Theme {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Light => "THEME_LIGHT",
            Self::Dark => "THEME_DARK",
            Self::System => "THEME_SYSTEM",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Theme {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "THEME_LIGHT",
            "THEME_DARK",
            "THEME_SYSTEM",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Theme;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "THEME_LIGHT" => Ok(Theme::Light),
                    "THEME_DARK" => Ok(Theme::Dark),
                    "THEME_SYSTEM" => Ok(Theme::System),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateEmailRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.email.is_empty() {
            len += 1;
        }
        if !self.temp_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.UpdateEmailRequest", len)?;
        if !self.email.is_empty() {
            struct_ser.serialize_field("email", &self.email)?;
        }
        if !self.temp_token.is_empty() {
            struct_ser.serialize_field("tempToken", &self.temp_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateEmailRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "email",
            "temp_token",
            "tempToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Email,
            TempToken,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "email" => Ok(GeneratedField::Email),
                            "tempToken" | "temp_token" => Ok(GeneratedField::TempToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateEmailRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.UpdateEmailRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateEmailRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut email__ = None;
                let mut temp_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Email => {
                            if email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("email"));
                            }
                            email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TempToken => {
                            if temp_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tempToken"));
                            }
                            temp_token__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdateEmailRequest {
                    email: email__.unwrap_or_default(),
                    temp_token: temp_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.UpdateEmailRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateLanguageRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.language != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.UpdateLanguageRequest", len)?;
        if self.language != 0 {
            let v = Language::try_from(self.language)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.language)))?;
            struct_ser.serialize_field("language", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateLanguageRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "language",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Language,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "language" => Ok(GeneratedField::Language),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateLanguageRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.UpdateLanguageRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateLanguageRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut language__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Language => {
                            if language__.is_some() {
                                return Err(serde::de::Error::duplicate_field("language"));
                            }
                            language__ = Some(map_.next_value::<Language>()? as i32);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdateLanguageRequest {
                    language: language__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.UpdateLanguageRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateNotificationsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.field.is_empty() {
            len += 1;
        }
        if self.value {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.UpdateNotificationsRequest", len)?;
        if !self.field.is_empty() {
            struct_ser.serialize_field("field", &self.field)?;
        }
        if self.value {
            struct_ser.serialize_field("value", &self.value)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateNotificationsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "field",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Field,
            Value,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "field" => Ok(GeneratedField::Field),
                            "value" => Ok(GeneratedField::Value),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateNotificationsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.UpdateNotificationsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateNotificationsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut field__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Field => {
                            if field__.is_some() {
                                return Err(serde::de::Error::duplicate_field("field"));
                            }
                            field__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdateNotificationsRequest {
                    field: field__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.UpdateNotificationsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdatePasswordRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.password.is_empty() {
            len += 1;
        }
        if !self.old_password.is_empty() {
            len += 1;
        }
        if !self.temp_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.UpdatePasswordRequest", len)?;
        if !self.password.is_empty() {
            struct_ser.serialize_field("password", &self.password)?;
        }
        if !self.old_password.is_empty() {
            struct_ser.serialize_field("oldPassword", &self.old_password)?;
        }
        if !self.temp_token.is_empty() {
            struct_ser.serialize_field("tempToken", &self.temp_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdatePasswordRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "password",
            "old_password",
            "oldPassword",
            "temp_token",
            "tempToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Password,
            OldPassword,
            TempToken,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "password" => Ok(GeneratedField::Password),
                            "oldPassword" | "old_password" => Ok(GeneratedField::OldPassword),
                            "tempToken" | "temp_token" => Ok(GeneratedField::TempToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdatePasswordRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.UpdatePasswordRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdatePasswordRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut password__ = None;
                let mut old_password__ = None;
                let mut temp_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Password => {
                            if password__.is_some() {
                                return Err(serde::de::Error::duplicate_field("password"));
                            }
                            password__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OldPassword => {
                            if old_password__.is_some() {
                                return Err(serde::de::Error::duplicate_field("oldPassword"));
                            }
                            old_password__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TempToken => {
                            if temp_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tempToken"));
                            }
                            temp_token__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdatePasswordRequest {
                    password: password__.unwrap_or_default(),
                    old_password: old_password__.unwrap_or_default(),
                    temp_token: temp_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.UpdatePasswordRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdatePrivacyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.field.is_empty() {
            len += 1;
        }
        if self.value {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.UpdatePrivacyRequest", len)?;
        if !self.field.is_empty() {
            struct_ser.serialize_field("field", &self.field)?;
        }
        if self.value {
            struct_ser.serialize_field("value", &self.value)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdatePrivacyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "field",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Field,
            Value,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "field" => Ok(GeneratedField::Field),
                            "value" => Ok(GeneratedField::Value),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdatePrivacyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.UpdatePrivacyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdatePrivacyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut field__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Field => {
                            if field__.is_some() {
                                return Err(serde::de::Error::duplicate_field("field"));
                            }
                            field__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdatePrivacyRequest {
                    field: field__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.UpdatePrivacyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateSecurityRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.field.is_empty() {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.UpdateSecurityRequest", len)?;
        if !self.field.is_empty() {
            struct_ser.serialize_field("field", &self.field)?;
        }
        if let Some(v) = self.value.as_ref() {
            match v {
                update_security_request::Value::TwoFactor(v) => {
                    struct_ser.serialize_field("twoFactor", v)?;
                }
                update_security_request::Value::MagicLink(v) => {
                    struct_ser.serialize_field("magicLink", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateSecurityRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "field",
            "two_factor",
            "twoFactor",
            "magic_link",
            "magicLink",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Field,
            TwoFactor,
            MagicLink,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "field" => Ok(GeneratedField::Field),
                            "twoFactor" | "two_factor" => Ok(GeneratedField::TwoFactor),
                            "magicLink" | "magic_link" => Ok(GeneratedField::MagicLink),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateSecurityRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.UpdateSecurityRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateSecurityRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut field__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Field => {
                            if field__.is_some() {
                                return Err(serde::de::Error::duplicate_field("field"));
                            }
                            field__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TwoFactor => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("twoFactor"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(update_security_request::Value::TwoFactor);
                        }
                        GeneratedField::MagicLink => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("magicLink"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(update_security_request::Value::MagicLink);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdateSecurityRequest {
                    field: field__.unwrap_or_default(),
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("client.v1.UpdateSecurityRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateThemeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.theme != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.UpdateThemeRequest", len)?;
        if self.theme != 0 {
            let v = Theme::try_from(self.theme)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.theme)))?;
            struct_ser.serialize_field("theme", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateThemeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "theme",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Theme,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "theme" => Ok(GeneratedField::Theme),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateThemeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.UpdateThemeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateThemeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut theme__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Theme => {
                            if theme__.is_some() {
                                return Err(serde::de::Error::duplicate_field("theme"));
                            }
                            theme__ = Some(map_.next_value::<Theme>()? as i32);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdateThemeRequest {
                    theme: theme__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.UpdateThemeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UploadAvatarRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.file.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.UploadAvatarRequest", len)?;
        if let Some(v) = self.file.as_ref() {
            struct_ser.serialize_field("file", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UploadAvatarRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "file",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            File,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "file" => Ok(GeneratedField::File),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UploadAvatarRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.UploadAvatarRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UploadAvatarRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut file__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::File => {
                            if file__.is_some() {
                                return Err(serde::de::Error::duplicate_field("file"));
                            }
                            file__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UploadAvatarRequest {
                    file: file__,
                })
            }
        }
        deserializer.deserialize_struct("client.v1.UploadAvatarRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UploadAvatarResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.UploadAvatarResponse", len)?;
        if !self.url.is_empty() {
            struct_ser.serialize_field("url", &self.url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UploadAvatarResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "url",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Url,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "url" => Ok(GeneratedField::Url),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UploadAvatarResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.UploadAvatarResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UploadAvatarResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Url => {
                            if url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UploadAvatarResponse {
                    url: url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.UploadAvatarResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for User {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.email.is_empty() {
            len += 1;
        }
        if self.avatar.is_some() {
            len += 1;
        }
        if self.settings.is_some() {
            len += 1;
        }
        if self.is_admin {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("client.v1.User", len)?;
        if !self.email.is_empty() {
            struct_ser.serialize_field("email", &self.email)?;
        }
        if let Some(v) = self.avatar.as_ref() {
            struct_ser.serialize_field("avatar", v)?;
        }
        if let Some(v) = self.settings.as_ref() {
            struct_ser.serialize_field("settings", v)?;
        }
        if self.is_admin {
            struct_ser.serialize_field("isAdmin", &self.is_admin)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for User {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "email",
            "avatar",
            "settings",
            "is_admin",
            "isAdmin",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Email,
            Avatar,
            Settings,
            IsAdmin,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "email" => Ok(GeneratedField::Email),
                            "avatar" => Ok(GeneratedField::Avatar),
                            "settings" => Ok(GeneratedField::Settings),
                            "isAdmin" | "is_admin" => Ok(GeneratedField::IsAdmin),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = User;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct client.v1.User")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<User, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut email__ = None;
                let mut avatar__ = None;
                let mut settings__ = None;
                let mut is_admin__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Email => {
                            if email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("email"));
                            }
                            email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Avatar => {
                            if avatar__.is_some() {
                                return Err(serde::de::Error::duplicate_field("avatar"));
                            }
                            avatar__ = map_.next_value()?;
                        }
                        GeneratedField::Settings => {
                            if settings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("settings"));
                            }
                            settings__ = map_.next_value()?;
                        }
                        GeneratedField::IsAdmin => {
                            if is_admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isAdmin"));
                            }
                            is_admin__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(User {
                    email: email__.unwrap_or_default(),
                    avatar: avatar__,
                    settings: settings__,
                    is_admin: is_admin__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("client.v1.User", FIELDS, GeneratedVisitor)
    }
}
