# HAB Predictor
The [Stanford Student Space Initiative](https://stanfordssi.org) regularly launches high altitude balloons. 
We want to know where they will go. 
Note that the readme below reflects the plans for the predictor, not what's currently implemented.  

## Public API
### /predict
This is where you may make a prediction. 

**Required Parameters**

| Parameter | Type    | Description                                                    |
|-----------|---------|----------------------------------------------------------------|
| lat       | float   | Launch latitude                                                |
| lon       | float   | Launch longitude                                               |
| altitude  | float   | Launch altitude, in meters                                     |
| time      | integer | UNIX timestamp (seconds since epoch) of the launch time        |
| profile   | string  | Which prediction profile to run. May be "standard" or "valbal" |

**Standard Profile Parameters**
Note: these parameters are required when profile is "standard"

| Parameter      | Type  | Description                                         |
|----------------|-------|-----------------------------------------------------|
| burst_altitude | float | Altitude at which balloon bursts, in meters         |
| ascent_rate    | float | Rate at which balloon ascends, in meters per second |
| descent_rate   | float | Rate at which balloon falls, in meters per second   |

**ValBal Parameters**
Note: these parameters are required when profile is "valbal"

| Parameter    | Type  | Description                                         |
|--------------|-------|-----------------------------------------------------|
| duration     | float | Minutes for which to run the prediction             |

**Response**
If successful, the API will respond with a 200 and a response of the following format:

```json
{
  "metadata": {
    "flightTime": "float in minutes"
  },
  "data": {
    "ascent": [
      {
        "lat": "float",
        "lon": "float",
        "altitude": "float",
        "time": "ISO String",
      }
    ],
    "burst": {
      "lat": "float",
      "lon": "float",
      "altitude": "float",
      "time": "ISO String",
    },
    "descent": [
      {
        "lat": "float",
        "lon": "float",
        "altitude": "float",
        "time": "ISO String",
      }
    ],
    "float": [
      {
        "lat": "float",
        "lon": "float",
        "altitude": "float",
        "time": "ISO String",
      }
    ]
  }
}
```
Note that `ascent`, `burst`, and `descent` will only be present for when profile is "standard", and `float` will only be present when profile is "valbal"

### /guidance
This is the core active guidance endpoint. In the initial version of the API, it will only support optimizing traveling east as fast as possible, but there are plans to let it navigate to a given point.  

**Required Parameters**

| Parameter    | Type    | Description                                                         |
|--------------|---------|---------------------------------------------------------------------|
| lat          | float   | Launch latitude                                                     |
| lon          | float   | Launch longitude                                                    |
| altitude     | float   | Launch altitude, in meters                                          |
| time         | integer | UNIX timestamp (seconds since epoch) of the launch time             |
| performance  | integer | Performance coefficient. Higher is more performant but takes longer |
| timeout      | integer | Max seconds to run guidance for. Limited to 60                      |
| altitude_res | float   | Granularity of altitude adjustments it assumes the payload can make in meters. Defaults to 500 |


**Response**
If successful, the API will respond with a 200 and a response of the following format:

```json
{
  "metadata": {},
  "adjustments": [
    {
        "lat": "float",
        "lon": "float",
        "altitude": "float",
        "time": "ISO String"
    }
  ]
}
```

## Installing
The simplest way to get it up and running will be to use the Docker Container once that's written. 
Otherwise, install Rails and Rust, then run `rails s`.  

## The nitty-gritty
### Profiles
The standard profile and the ValBal profile are fundamentally different.
The standard profile represents and ordinary high altitude balloon: one that ascends to approximately 100,000 feet, pops, and falls back down.
If you aren't part of the Stanford Student Space Initiative, you'll almost certainly be using the standard profile.

The ValBal profile, on the other hand, is used for modeling our altitude control system. 

### Downloader
It currently downloads the GFS predictions from NOAA. 
You can see the datasets at [https://nomads.ncdc.noaa.gov/data/gfs4/](https://nomads.ncdc.noaa.gov/data/gfs4/). 
This model runs four times per day, at 00, 06, 12, and 18 UTC.
 
There are also plans to support the ECMWF model, as that tends to be more accurate, especially over mountains.
However, this data is not free, and so has not yet been integrated.

### Active Guidance
One of the core features of our novel altitude stabilization platform, ValBal, is that it’s capable of flying at whatever altitude we tell it to, and adjusting that altitude mid-flight. 
Since there are different wind patterns at different altitudes, by strategically adjusting altitudes the balloon can optimize ground distance or even aim for a specific location. 
For example, the balloon might fly at 13km to travel west, then ascend to 15km briefly to steer north toward its final destination.
