
namespace :guidance do
  task :test => [:environment] do
    result = Predictor.guidance(
        lat: 36.8491253,
        lon: -121.4342394,
        altitude: 12000,
        time: 1.hour.from_now,

        duration: 3.days,
        timeout: 30.seconds,
        compare_with_naive: true
    )

    test_output result

    puts
    puts "Got to longitude #{active['longitude']} (only #{naive['longitude']} naively) by #{time}"
  end

  task :test_greenland => [:environment] do

    destination = {
        latitude: 66.364224,
        longitude: -38.1470757
    }

    result = Predictor.guidance(
        lat: 64.1791025,
        lon: -51.7418292,
        altitude: 12000,
        time: 1.hour.from_now,

        duration: 3.days,
        timeout: 10.seconds,
        compare_with_naive: true,

        guidance_type: 'destination',
        destination_lat: destination[:latitude],
        destination_lon: destination[:longitude]
    )

    test_output result

    naive = (result['naive'].last || {}).symbolize_keys
    active = (result['positions'].last || {}).symbolize_keys

    naive_distance = distance(destination, naive)
    guided_distance = distance(destination, active)

    puts "Got within #{guided_distance.round}km (#{naive_distance.round}km naively)"

  end

  # distance in kilometers between coordinates
  def distance(from, to, earth_radius: 6_371)
    rad_per_deg = Math::PI / 180

    lat_from = from[:latitude].to_f * rad_per_deg
    lon_from = from[:lon].to_f * rad_per_deg
    lat_to = to[:latitude].to_f * rad_per_deg
    lon_to = to[:lon].to_f * rad_per_deg

    lat_delta = lat_to - lat_from
    lon_delta = lon_to - lon_from

    angle = 2 * Math.asin(Math.sqrt(Math.sin(lat_delta / 2)**2)) +
        Math.cos(lat_from) * Math.cos(lat_to) * Math.sin(lon_delta / 2**2)

    angle * earth_radius
  end


  def test_output(result)
    # puts JSON.pretty_generate result
    # puts "\n\n\n"


    naive = (result['naive'].last || {})
    active = (result['positions'].last || {})
    time = active['time']

    puts 'Final position for active guidance'
    puts JSON.pretty_generate active

    puts
    puts 'Final position for naive'
    puts JSON.pretty_generate naive

    puts
    puts 'Metadata'
    puts JSON.pretty_generate result['metadata']
  end

end
