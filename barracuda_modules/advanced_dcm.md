# The Advanced Door Control Module
The Advanced Door Control Module implements a fully configurable control for an arbitrary number of passageways on barracuda. Each passageway (logically) consists of a number of IOs that are orchestrated to generate the behavior required by the passageway.

## Components
### Access Point
The access point is used to assign an accesspoint ID to the inner or outer side of a given passageway. The used ID is a logical ID (i.e. not a SUD!).
### Electric Door Opener
The Electric Door Opener is an __output__ component that will trigger a digital output of the device whenever the passageway receives a door open request (this request is e.g. triggered, when an identification token was presented to generic_whitelist, that has access at the given access point). 

Configurable Values:
SwitchTime (ms): This parameter denotes the maximum number of milliseconds the door will stay released. Default: 5000.
### Access Granted Relais
### Alarm Relay
The alarm relay is an __output__ component that is triggered whenever abnormal behavior is detected by the framecontact. 
### Frame Contact
The frame contact is an __input__ component that is responsible for sensing the state of the door (i.e. is it closed or open). The FC will generate alarms if it detects abnormal behavior (see "Alarmsystem").
### Door Opener Key
### Door Handle
### Blocking Contact
The blocking contact is an  __input__ component, that is used to completely shut off access through the door. As soon as the blocking contact is detected to be "logically high" all access requests for the door will be rejected.
### Release Contact
### Interlock

## Doorflows

## Doorprofiles
### Door Open Profile
### Auto Release

## Operation
### Default Operation

### Alarmsystem
#### Alert (Door open warning)
After the door was opened (authorized!), the FC will check, that the door is closed within a given limit. If it does not detect a closed door during that time it will trigger an alert signalisation that is intended to notify bystanding users that the door is not closed and should be closed. This alert is local to the device and not propagated to the controling system.
#### Alarm (Door open too long, silent)
After an alert was triggerd and if noone reacted a silent alarm is triggered that is immediately propagated to the controling system.
#### Alarm (Door forced open, silent)
If the FC detects an unauthorized access (i.e. the door was opened without there being an access granted event) it will immediately trigger this silent alarm, which is propagated to the controling system.


## Configuration

## Commands
#### Release Once
#### Permanent Release
#### Block

## Acknowledgements
The ADCM uses some terminology that was originally coined by dormakaba's exos system, especially with regard to the component names.