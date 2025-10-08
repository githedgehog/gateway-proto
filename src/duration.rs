// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::google::protobuf::Duration as ProtoDuration;
use std::time::Duration as StdDuration;

#[derive(thiserror::Error, Debug)]
pub enum DurationConversionError {
    #[error("Duration cannot be negative ({0} seconds, {1} nanoseconds)")]
    NegativeDuration(i64, i32),
}

impl TryFrom<&ProtoDuration> for StdDuration {
    type Error = DurationConversionError;

    fn try_from(proto_duration: &ProtoDuration) -> Result<Self, Self::Error> {
        let seconds = proto_duration.seconds;
        let nanos = proto_duration.nanos;

        let nanos_as_seconds = nanos / 1_000_000_000;
        let nanos_leftover = nanos - nanos_as_seconds * 1_000_000_000;
        let new_seconds = seconds + i64::from(nanos_as_seconds);

        assert!(nanos_leftover.abs() < 1_000_000_000);

        if new_seconds < 0 {
            return Err(DurationConversionError::NegativeDuration(seconds, nanos));
        }

        let (seconds_u64, nanos_u32) = if nanos_leftover < 0 {
            if new_seconds < 1 {
                return Err(DurationConversionError::NegativeDuration(seconds, nanos));
            }
            let new_seconds = new_seconds - 1;
            let nanos_leftover = nanos_leftover + 1_000_000_000;

            (
                u64::try_from(new_seconds).expect("Failed to convert Proto Duration seconds to std::time::Duration with negative nanos_leftover (should never happen"),
                u32::try_from(nanos_leftover).expect("Failed to convert Proto Duration nanos to std::time::Duration with negative nanos_leftover (should never happen"),
            )
        } else {
            (
                u64::try_from(new_seconds).expect("Failed to convert Proto Duration seconds to std::time::Duration (should never happen)"),
                u32::try_from(nanos_leftover).expect("Failed to convert Proto Duration nanos to std::time::Duration (should never happen)"),
            )
        };

        Ok(StdDuration::new(seconds_u64, nanos_u32))
    }
}

impl TryFrom<ProtoDuration> for StdDuration {
    type Error = DurationConversionError;

    fn try_from(proto_duration: ProtoDuration) -> Result<Self, Self::Error> {
        StdDuration::try_from(&proto_duration)
    }
}

impl TryFrom<&StdDuration> for ProtoDuration {
    type Error = std::num::TryFromIntError;

    fn try_from(std_duration: &StdDuration) -> Result<Self, Self::Error> {
        let seconds = i64::try_from(std_duration.as_secs())?;
        let nanos = i32::try_from(std_duration.subsec_nanos())?;
        Ok(ProtoDuration { seconds, nanos })
    }
}

impl TryFrom<StdDuration> for ProtoDuration {
    type Error = std::num::TryFromIntError;

    fn try_from(std_duration: StdDuration) -> Result<Self, Self::Error> {
        Self::try_from(&std_duration)
    }
}

#[cfg(test)]
mod tests {
    use crate::google::protobuf::Duration as ProtoDuration;

    #[test]
    fn test_proto_to_std_duration_conversion_bolero() {
        bolero::check!()
            .with_type::<ProtoDuration>()
            .for_each(|proto_duration| {
                let result = std::time::Duration::try_from(proto_duration);
                match result {
                    Ok(duration) => {
                        // This check is not complete, but easy to check in this case
                        if proto_duration.seconds > 0 && proto_duration.nanos > 0 {
                            assert_eq!(
                                duration,
                                std::time::Duration::new(
                                    u64::try_from(proto_duration.seconds).unwrap(),
                                    u32::try_from(proto_duration.nanos).unwrap()
                                )
                            );
                        }
                    }
                    Err(_) => {
                        // This is not a full check that the error is valid, but one of these must be true for there to be a conversion error
                        assert!(proto_duration.seconds < 0 || proto_duration.nanos < 0);
                    }
                }
            });
    }

    #[test]
    fn test_proto_to_std_duration_conversion_negative_nanos() {
        let proto_duration = ProtoDuration {
            seconds: 10,
            nanos: -1,
        };
        let result = std::time::Duration::try_from(proto_duration);
        assert_eq!(result.unwrap(), std::time::Duration::new(9, 999_999_999));

        let proto_duration = ProtoDuration {
            seconds: 10,
            nanos: -1_000_000_001,
        };
        let result = std::time::Duration::try_from(proto_duration);
        assert_eq!(result.unwrap(), std::time::Duration::new(8, 999_999_999));
    }

    #[test]
    fn test_proto_to_std_duration_conversion_negative_seconds() {
        let proto_duration = ProtoDuration {
            seconds: -1,
            nanos: 1_000_000_001,
        };
        let result = std::time::Duration::try_from(proto_duration);
        assert_eq!(result.unwrap(), std::time::Duration::new(0, 1));

        let proto_duration = ProtoDuration {
            seconds: -1,
            nanos: 2_000_000_000,
        };
        let result = std::time::Duration::try_from(proto_duration);
        assert_eq!(result.unwrap(), std::time::Duration::new(1, 0));
    }

    #[test]
    fn test_std_to_proto_duration_conversion_bolero() {
        bolero::check!()
            .with_type::<std::time::Duration>()
            .for_each(|duration| {
                let proto_duration_result = ProtoDuration::try_from(duration);
                match proto_duration_result {
                    Err(_) => {
                        assert!(
                            i64::try_from(duration.as_secs()).is_err()
                                || i32::try_from(duration.subsec_nanos()).is_err()
                        );
                    }
                    Ok(proto_duration) => {
                        assert_eq!(
                            u64::try_from(proto_duration.seconds).unwrap(),
                            duration.as_secs()
                        );
                        assert_eq!(
                            u32::try_from(proto_duration.nanos).unwrap(),
                            duration.subsec_nanos()
                        );
                    }
                }
            });
    }
}
