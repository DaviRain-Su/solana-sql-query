use super::subscriber_email::SubscriberEmail;
use super::subscriber_name::SubscriberName;
use crate::router::FormData;

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = anyhow::Error;

    fn try_from(values: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(values.email)?;
        let name = SubscriberName::parse(values.name)?;
        Ok(Self { email, name })
    }
}
