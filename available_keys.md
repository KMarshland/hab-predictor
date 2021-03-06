# Available keys
These are variables we have data for. 
Which ones are available depends on the altitude.
The first word is the "shortName", which is ideal for querying on; the rest is the full name.

## Keys available at level 80
This corresponds to about 18km.

Generated with `grib_get -p shortName,name -w level=80 ~/Programming/ssi/prediction/data/gfs_4_20170923_0000_216.grb2`

```text
pres Pressure
q Specific humidity
t Temperature
u U component of wind
v V component of wind
```

## Keys available at level 0
Generated with `grib_get -p shortName,name -w level=0 ~/Programming/ssi/prediction/data/gfs_4_20170923_0000_216.grb2`

```text
4lftx Best (4-layer) lifted index
acpcp Convective precipitation (water)
al Albedo
cape Convective available potential energy
cfrzr Categorical freezing rain
ci Sea-ice cover
cicep Categorical ice pellets
cin Convective inhibition
cpofp Percent frozen precipitation
cprat Convective precipitation rate
crain Categorical rain
csnow Categorical snow
cwat Cloud water
cwork Cloud work function
dlwrf Downward long-wave radiation flux
dswrf Downward short-wave radiation flux
fldcp Field Capacity
gflux Ground heat flux
gh Geopotential Height
gust Wind speed (gust)
hindex Haines Index
hpbl Planetary boundary layer height
icaht ICAO Standard Atmosphere reference height
landn Land-sea coverage (nearest neighbor) [land=1,sea=0]
lftx Surface lifted index
lhtfl Latent heat net flux
lsm Land-sea mask
mslet MSLP (Eta model reduction)
orog Orography
pevpr Potential evaporation rate
prate Precipitation rate
pres Pressure
prmsl Pressure reduced to MSL
pwat Precipitable water
r Relative humidity
sde Snow depth
sdwe Water equivalent of accumulated snow depth
shtfl Sensible heat net flux
soilw Volumetric soil moisture content
sp Surface pressure
st Soil Temperature
SUNSD Sunshine Duration
t Temperature
tcc Total Cloud Cover
tozne Total ozone
tp Total Precipitation
u U component of wind
u-gwd Zonal flux of gravity wave stress
uflx Momentum flux, u component
ulwrf Upward long-wave radiation flux
uswrf Upward short-wave radiation flux
v V component of wind
v-gwd Meridional flux of gravity wave stress
vflx Momentum flux, v component
vis Visibility
VRATE Ventilation Rate
vwsh Vertical speed shear
watr Water runoff
wilt Wilting Point
```

## Keys available at level 1
Generated with `grib_get -p shortName,name -w level=1 ~/Programming/ssi/prediction/data/gfs_4_20170923_0000_216.grb2`

```text
gh Geopotential Height
o3mr Ozone mixing ratio
pt Potential temperature
r Relative humidity
soilw Volumetric soil moisture content
st Soil Temperature
t Temperature
u U component of wind
v V component of wind
w Vertical velocity
```

## All keys 

Generated with `grib_get -p shortName,name ~/Programming/ssi/prediction/data/gfs_4_20170923_0000_216.grb2`

```text
10u 10 metre U wind component
10v 10 metre V wind component
2d 2 metre dewpoint temperature
2r Surface air relative humidity
2t 2 metre temperature
4lftx Best (4-layer) lifted index
5wavh 5-wave geopotential height
absv Absolute vorticity
acpcp Convective precipitation (water)
al Albedo
aptmp Apparent temperature
cape Convective available potential energy
cfrzr Categorical freezing rain
ci Sea-ice cover
cicep Categorical ice pellets
cin Convective inhibition
clwmr Cloud mixing ratio
cpofp Percent frozen precipitation
cprat Convective precipitation rate
crain Categorical rain
csnow Categorical snow
cwat Cloud water
cwork Cloud work function
dlwrf Downward long-wave radiation flux
dswrf Downward short-wave radiation flux
fldcp Field Capacity
gflux Ground heat flux
gh Geopotential Height
gust Wind speed (gust)
hindex Haines Index
hlcy Storm relative helicity
hpbl Planetary boundary layer height
icaht ICAO Standard Atmosphere reference height
ICSEV Icing severity
landn Land-sea coverage (nearest neighbor) [land=1,sea=0]
lftx Surface lifted index
lhtfl Latent heat net flux
lsm Land-sea mask
mslet MSLP (Eta model reduction)
o3mr Ozone mixing ratio
orog Orography
pevpr Potential evaporation rate
plpl Pressure of level from which parcel was lifted
prate Precipitation rate
pres Pressure
prmsl Pressure reduced to MSL
pt Potential temperature
pwat Precipitable water
q Specific humidity
r Relative humidity
sde Snow depth
sdwe Water equivalent of accumulated snow depth
shtfl Sensible heat net flux
soilw Volumetric soil moisture content
sp Surface pressure
st Soil Temperature
SUNSD Sunshine Duration
t Temperature
tcc Total Cloud Cover
tmax Maximum temperature
tmin Minimum temperature
tozne Total ozone
tp Total Precipitation
u U component of wind
u-gwd Zonal flux of gravity wave stress
uflx Momentum flux, u component
ulwrf Upward long-wave radiation flux
ustm U-component storm motion
uswrf Upward short-wave radiation flux
v V component of wind
v-gwd Meridional flux of gravity wave stress
vflx Momentum flux, v component
vis Visibility
VRATE Ventilation Rate
vstm V-component storm motion
vwsh Vertical speed shear
w Vertical velocity
watr Water runoff
wilt Wilting Point
```