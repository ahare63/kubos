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

use super::*;

#[test]
fn debug_telem_good() {
    let mock = mock_new!();

    mock.get_activation_count
        .return_value_for(KANTSAnt::Ant1, Ok(1));
    mock.get_activation_time
        .return_value_for(KANTSAnt::Ant1, Ok(11));
    mock.get_activation_count
        .return_value_for(KANTSAnt::Ant2, Ok(2));
    mock.get_activation_time
        .return_value_for(KANTSAnt::Ant2, Ok(22));
    mock.get_activation_count
        .return_value_for(KANTSAnt::Ant3, Ok(3));
    mock.get_activation_time
        .return_value_for(KANTSAnt::Ant3, Ok(33));
    mock.get_activation_count
        .return_value_for(KANTSAnt::Ant4, Ok(4));
    mock.get_activation_time
        .return_value_for(KANTSAnt::Ant4, Ok(44));
    let service = service_new!(mock);

    let query = r#"
        {
            telemetry {
                debug {
                     ant1ActivationCount,
                     ant1ActivationTime,
                     ant2ActivationCount,
                     ant2ActivationTime,
                     ant3ActivationCount,
                     ant3ActivationTime,
                     ant4ActivationCount,
                     ant4ActivationTime,
                }
            }
        }"#;

    let expected = json!({
            "telemetry": {
                "debug": {
                     "ant1ActivationCount": 1,
                     "ant1ActivationTime": 11,
                     "ant2ActivationCount": 2,
                     "ant2ActivationTime": 22,
                     "ant3ActivationCount": 3,
                     "ant3ActivationTime": 33,
                     "ant4ActivationCount": 4,
                     "ant4ActivationTime": 44,
                }
            }
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn debug_telem_bad() {
    let mock = mock_new!();

    mock.get_activation_count
        .return_value(Err(AntsError::GenericError));
    mock.get_activation_time
        .return_value(Err(AntsError::GenericError));

    let service = service_new!(mock);

    let query = r#"
        {
            telemetry {
                debug {
                     ant1ActivationCount,
                     ant1ActivationTime,
                     ant2ActivationCount,
                     ant2ActivationTime,
                     ant3ActivationCount,
                     ant3ActivationTime,
                     ant4ActivationCount,
                     ant4ActivationTime,
                }
            }
        }"#;

    let expected = json!({
            "telemetry": {
                "debug": {
                     "ant1ActivationCount": 0,
                     "ant1ActivationTime": 0,
                     "ant2ActivationCount": 0,
                     "ant2ActivationTime": 0,
                     "ant3ActivationCount": 0,
                     "ant3ActivationTime": 0,
                     "ant4ActivationCount": 0,
                     "ant4ActivationTime": 0,
                }
            }
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn nominal_telem_good() {
    let mock = mock_new!();

    let nominal = AntsTelemetry {
        raw_temp: 15,
        uptime: 35,
        deploy_status: DeployStatus {
            sys_armed: true,
            ant_1_active: true,
            ant_4_not_deployed: false,
            ..Default::default()
        },
    };
    mock.get_system_telemetry.return_value(Ok(nominal.clone()));

    let service = service_new!(mock);

    let query = r#"
        {
            telemetry {
                nominal {
                     rawTemp,
                     uptime,
                     sysBurnActive,
                     sysIgnoreDeploy,
                     sysArmed,
                     ant1NotDeployed,
                     ant1StoppedTime,
                     ant1Active,
                     ant2NotDeployed,
                     ant2StoppedTime,
                     ant2Active,
                     ant3NotDeployed,
                     ant3StoppedTime,
                     ant3Active,
                     ant4NotDeployed,
                     ant4StoppedTime,
                     ant4Active
                }
            }
        }"#;

    let expected = json!({
            "telemetry": {
                "nominal": {
                     "rawTemp": 15,
                     "uptime": 35,
                     "sysBurnActive": false,
                     "sysIgnoreDeploy": false,
                     "sysArmed": true,
                     "ant1NotDeployed": false,
                     "ant1StoppedTime": false,
                     "ant1Active": true,
                     "ant2NotDeployed": false,
                     "ant2StoppedTime": false,
                     "ant2Active": false,
                     "ant3NotDeployed": false,
                     "ant3StoppedTime": false,
                     "ant3Active": false,
                     "ant4NotDeployed": false,
                     "ant4StoppedTime": false,
                     "ant4Active": false
                }
            }
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn nominal_telem_nondefault() {
    let mock = mock_new!();

    let nominal = AntsTelemetry {
        raw_temp: 15,
        uptime: 35,
        deploy_status: DeployStatus {
            sys_burn_active: true,
            sys_ignore_deploy: true,
            sys_armed: true,
            ant_1_not_deployed: true,
            ant_1_stopped_time: true,
            ant_1_active: true,
            ant_2_not_deployed: true,
            ant_2_stopped_time: true,
            ant_2_active: true,
            ant_3_not_deployed: true,
            ant_3_stopped_time: true,
            ant_3_active: true,
            ant_4_not_deployed: true,
            ant_4_stopped_time: true,
            ant_4_active: true,
        },
    };
    mock.get_system_telemetry.return_value(Ok(nominal.clone()));

    let service = service_new!(mock);

    let query = r#"
        {
            telemetry {
                nominal {
                     rawTemp,
                     uptime,
                     sysBurnActive,
                     sysIgnoreDeploy,
                     sysArmed,
                     ant1NotDeployed,
                     ant1StoppedTime,
                     ant1Active,
                     ant2NotDeployed,
                     ant2StoppedTime,
                     ant2Active,
                     ant3NotDeployed,
                     ant3StoppedTime,
                     ant3Active,
                     ant4NotDeployed,
                     ant4StoppedTime,
                     ant4Active
                }
            }
        }"#;

    let expected = json!({
            "telemetry": {
                "nominal": {
                     "rawTemp": 15,
                     "uptime": 35,
                     "sysBurnActive": true,
                     "sysIgnoreDeploy": true,
                     "sysArmed": true,
                     "ant1NotDeployed": true,
                     "ant1StoppedTime": true,
                     "ant1Active": true,
                     "ant2NotDeployed": true,
                     "ant2StoppedTime": true,
                     "ant2Active": true,
                     "ant3NotDeployed": true,
                     "ant3StoppedTime": true,
                     "ant3Active": true,
                     "ant4NotDeployed": true,
                     "ant4StoppedTime": true,
                     "ant4Active": true
                }
            }
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn nominal_telem_bad() {
    let mock = mock_new!();

    mock.get_system_telemetry
        .return_value(Err(AntsError::GenericError));

    let service = service_new!(mock);

    let query = r#"
        {
            telemetry {
                nominal {
                     rawTemp,
                     uptime,
                     sysBurnActive,
                     sysIgnoreDeploy,
                     sysArmed,
                     ant1NotDeployed,
                     ant1StoppedTime,
                     ant1Active,
                     ant2NotDeployed,
                     ant2StoppedTime,
                     ant2Active,
                     ant3NotDeployed,
                     ant3StoppedTime,
                     ant3Active,
                     ant4NotDeployed,
                     ant4StoppedTime,
                     ant4Active
                }
            }
        }"#;

    let expected = json!({
            "telemetry": {
                "nominal": {
                     "rawTemp": 0,
                     "uptime": 0,
                     "sysBurnActive": false,
                     "sysIgnoreDeploy": false,
                     "sysArmed": false,
                     "ant1NotDeployed": false,
                     "ant1StoppedTime": false,
                     "ant1Active": false,
                     "ant2NotDeployed": false,
                     "ant2StoppedTime": false,
                     "ant2Active": false,
                     "ant3NotDeployed": false,
                     "ant3StoppedTime": false,
                     "ant3Active": false,
                     "ant4NotDeployed": false,
                     "ant4StoppedTime": false,
                     "ant4Active": false
                }
            }
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn telemetry_full() {
    let mock = mock_new!();

    let nominal = AntsTelemetry {
        raw_temp: 15,
        uptime: 35,
        deploy_status: DeployStatus {
            sys_armed: true,
            ant_1_active: true,
            ant_4_not_deployed: false,
            ..Default::default()
        },
    };

    mock.get_system_telemetry.return_value(Ok(nominal.clone()));

    mock.get_activation_count
        .return_value_for(KANTSAnt::Ant1, Ok(1));
    mock.get_activation_time
        .return_value_for(KANTSAnt::Ant1, Ok(11));
    mock.get_activation_count
        .return_value_for(KANTSAnt::Ant2, Ok(2));
    mock.get_activation_time
        .return_value_for(KANTSAnt::Ant2, Ok(22));
    mock.get_activation_count
        .return_value_for(KANTSAnt::Ant3, Ok(3));
    mock.get_activation_time
        .return_value_for(KANTSAnt::Ant3, Ok(33));
    mock.get_activation_count
        .return_value_for(KANTSAnt::Ant4, Ok(4));
    mock.get_activation_time
        .return_value_for(KANTSAnt::Ant4, Ok(44));

    let service = service_new!(mock);

    let query = r#"
        {
            telemetry {
                debug {
                     ant1ActivationCount,
                     ant1ActivationTime,
                     ant2ActivationCount,
                     ant2ActivationTime,
                     ant3ActivationCount,
                     ant3ActivationTime,
                     ant4ActivationCount,
                     ant4ActivationTime,
                },
                nominal {
                     rawTemp,
                     uptime,
                     sysBurnActive,
                     sysIgnoreDeploy,
                     sysArmed,
                     ant1NotDeployed,
                     ant1StoppedTime,
                     ant1Active,
                     ant2NotDeployed,
                     ant2StoppedTime,
                     ant2Active,
                     ant3NotDeployed,
                     ant3StoppedTime,
                     ant3Active,
                     ant4NotDeployed,
                     ant4StoppedTime,
                     ant4Active
                }
            }
        }"#;

    let expected = json!({
            "telemetry": {
                "debug": {
                         "ant1ActivationCount": 1,
                         "ant1ActivationTime": 11,
                         "ant2ActivationCount": 2,
                         "ant2ActivationTime": 22,
                         "ant3ActivationCount": 3,
                         "ant3ActivationTime": 33,
                         "ant4ActivationCount": 4,
                         "ant4ActivationTime": 44,
                },
                "nominal": {
                     "rawTemp": 15,
                     "uptime": 35,
                     "sysBurnActive": false,
                     "sysIgnoreDeploy": false,
                     "sysArmed": true,
                     "ant1NotDeployed": false,
                     "ant1StoppedTime": false,
                     "ant1Active": true,
                     "ant2NotDeployed": false,
                     "ant2StoppedTime": false,
                     "ant2Active": false,
                     "ant3NotDeployed": false,
                     "ant3StoppedTime": false,
                     "ant3Active": false,
                     "ant4NotDeployed": false,
                     "ant4StoppedTime": false,
                     "ant4Active": false
                }
            }
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}
