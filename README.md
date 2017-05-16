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
        "time": "float: UNIX timestamp (seconds since epoch)",
      }
    ],
    "burst": {
      "lat": "float",
      "lon": "float",
      "time": "float: UNIX timestamp (seconds since epoch)",
    },
    "descent": [
      {
        "lat": "float",
        "lon": "float",
        "time": "float: UNIX timestamp (seconds since epoch)",
      }
    ],
    "float": [
      {
        "lat": "float",
        "lon": "float",
        "time": "float: UNIX timestamp (seconds since epoch)",
      }
    ]
  }
}
```
Note that `ascent`, `burst`, and `descent` will only be present for when profile is "standard", and `float` will only be present when profile is "valbal"

### /guidance
This is the core active guidance endpoint. In the initial version of the API, it will only support optimizing traveling east as fast as possible, but there are plans to let it navigate to a given point.  

**Required Parameters**
| Parameter   | Type    | Description                                                         |
|-------------|---------|---------------------------------------------------------------------|
| lat         | float   | Launch latitude                                                     |
| lon         | float   | Launch longitude                                                    |
| altitude    | float   | Launch altitude, in meters                                          |
| time        | integer | UNIX timestamp (seconds since epoch) of the launch time             |
| performance | integer | Performance coefficient. Higher is more performant but takes longer |
| timeout     | integer | Max seconds to run guidance for. Limited to 60                      |

## Installing
The simplest way to get it up and running will be to use the Docker Container once that's written. 
Otherwise, install Rails and Rust, then run `rails s`.  

## The nitty-gritty
### Profiles
### Downloader
### Active Guidance