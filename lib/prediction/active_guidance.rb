#monkey patch the math methods
class Numeric
  (Math.methods - Module.methods - ["hypot", "ldexp"]).each do |method|
    define_method method do
      Math.send method, self
    end
  end
end

require_relative './djikstra'
require_relative './node'

module Prediction
  class << self

    def naive(params={})
      params = fix_params params
      segments = params[:segments] || 20

      result = [Node.from_pos(
          params[:start][:lat].to_f, params[:start][:lon].to_f, params[:start][:altitude].to_f, params[:start][:time]
      )]

      while result.size < segments
        prediction = result.last.prediction

        break if prediction.is_a? Hash

        last = prediction.last.with_indifferent_access

        lat = last[:latitude].to_f
        lon = last[:longitude].to_f
        start_alt = last[:altitude].to_f
        time = last[:datetime]

        result.push Node.from_pos(lat, lon, start_alt, time)
      end
      result
    end

    #goes as far east as possible
    def maximize_distance(params = {})
      params = fix_params params

      optimize_for_location({
                                start: params[:start],
                                finish: {
                                    lat: params[:start][:lat],
                                    lon: 175 #somewhere halfway across the pacific
                                },
                                timeout: params[:timeout],
                                performance: params[:performance],
                                use_faa: params[:use_faa],
                                check_countries: params[:countries]
                            })
    end

    def fix_params(params)
      params = params.with_indifferent_access

      if params[:start].nil?
        last = Transmission.order('transmit_time ASC').last
        params[:start] = {
            lat: last.latitude,
            lon: last.longitude,
            altitude: last.altitude
        }
      end

      params
    end

    #tries to go as far east as possible
    def optimize_for_location(params)
      flight_floor = params[:flight_floor] || 5000 #meters
      flight_ceiling = params[:flight_ceiling] || 60000 #meters

      #verify parameters
      params = params.with_indifferent_access

      raise 'No start altitude (params[:start][:altitude])' unless params[:start][:altitude].present?
      raise 'No start latitude (params[:start][:lat])' unless params[:start][:lat].present?
      raise 'No start longitude (params[:start][:lon])' unless params[:start][:lon].present?
      raise 'No start time (params[:start][:time])' unless params[:start][:time].present?

      raise 'No finish latitude (params[:finish][:lat])' unless params[:finish][:lat].present?
      raise 'No finish longitude (params[:finish][:lon])' unless params[:finish][:lon].present?

      # creates a filter lambda to check that any path passes certain conditions
      filter = lambda {|node, previous|
        node.altitude < flight_ceiling &&
        node.altitude > flight_floor &&
        # (!x || y) checks y only if x is true
        (!params[:use_faa] || node.faa_zones_approx(previous)) && #no restricted zones
        (!params[:check_countries] || node.countries(previous))
      }

      #run the search with the provided start, duration, and performance factor
      greedy_search({
                        start: Node.from_pos(
                            params[:start][:lat].to_f, params[:start][:lon].to_f,
                            params[:start][:altitude].to_f, params[:start][:time]
                        ),
                        finish: Node.from_pos(
                            params[:finish][:lat].to_f, params[:finish][:lon].to_f,
                            params[:start][:altitude].to_f, params[:start][:time]
                        ),
                        movement_cost: lambda {|current|
                          #vent / ballast costs are proportional to the square of the altitude change
                          (current.parent.altitude - current.altitude).abs/1200000.0
                        },
                        heuristic: lambda {|current|
                          ((current.parent.lon - current.lon) / Prediction::time_variance)
                        },
                        neighbors: lambda {|current|
                          current.neighbors(filter)
                        },
                        build_from_finish: lambda{|finish|
                          finish.build_chain
                        },
                        timeout: params[:timeout],
                        performance: params[:performance]
                    })

    end

    # checks whether a polygon intersects a line
    def intersects_polygon?(coordinates, lat_start, lon_start, lat_end, lon_end)
      prev = coordinates.last
      does_intersect = false

      # checks if the line segment between start and end coordinates intersects any line segments in the polygon
      coordinates.each do |coord|
        a_lat = prev['lat']
        a_lon = prev['lon']
        b_lat = coord['lat']
        b_lon = coord['lon']

        # line segment intersection as described by http://stackoverflow.com/a/565282
        cmp_x, cmp_y = (lat_start - a_lat), (lon_start - a_lon)
        r_x, r_y = (b_lat - a_lat), (b_lon - a_lon)
        s_x, s_y = (lat_end - lat_start), (lon_end - lon_start)
     
        cmpxr = (cmp_x * r_y) - (cmp_y * r_x)
        cmpxs = (cmp_x * s_y) - (cmp_y * s_x)
        rxs = (r_x * s_y) - (r_y * s_x)
     
        rxsr = 1.0 / rxs
        t = cmpxs * rxsr
        u = cmpxr * rxsr
     
        if ((t >= 0.0) && (t <= 1.0) && (u >= 0.0) && (u <= 1.0))
          does_intersect = true
        end


        prev = coord
      end

      return does_intersect
    end

    # checks if a path passes through any countries we don't want it to
    def find_countries(lat_start, lon_start, lat_end, lon_end)

      countries = JSON(File.read(Rails.root.join('lib','prediction','countries.json')))
      countries.each do |country|
        if intersects_polygon?(country['coordinates'], lat_start, lon_start, lat_end, lon_end)
          return false
        end
      end

      return true
    end

    #checks if the given coordinates are in an FAA zone; if so, returns the zone
    def find_faa_zones(lat_start, lon_start, lat_end, lon_end)

      # start_time = DateTime.now

      #read and parse out the faa zones
      unless @zones.present?
        @zones = JSON(File.read(Rails.root.join('lib', 'prediction', 'faa_zones.json').to_s))
        @zones.map! do |zone|

          # we only care about restricted zones
          if !(zone['restriction'].present?)
            next
          end

          if zone['boundaries']['shape'] == 'circle'
            center = zone['boundaries']['center']
            radius = zone['radius']
            zone = nil if center.nil? || radius.nil?

            if zone.present?
              center['lat'] = parse_coordinate center['lat']
              center['lon'] = parse_coordinate center['lon']

              #convert the radius to meters
              number = radius.gsub(',', '').to_f
              case radius
                when /(NM)|(nautical-mile)/i
                  number *= 1852
                when /mile/i
                  number *= 1609.34
                when /(foot)|(feet)|(ft)/i
                  number *= 0.3048
                when /(kilometer)|(km)/i
                  number *= 1000
              end

              zone['boundaries']['center'] = center
              zone['radius'] = number
            end
          elsif zone['boundaries']['shape'] == 'polygon'
            zone['boundaries']['coordinates'] = zone['boundaries']['coordinates'].select{|a|
              a['lat'].present? && a['lon'].present?
            }.map{|a|
              a['lat'] = parse_coordinate a['lat']
              a['lon'] = parse_coordinate a['lon']
              a
            }
          end
          zone
        end
        @zones.select!{|zone|
          zone.present?
        }
      end

      @zones.each do |zone|
        if zone['boundaries']['shape'] == 'circle'
          center = zone['boundaries']['center']
          radius = zone['radius']

          c_lat = center['lat']
          c_lon = center['lon']

          area2 = ((lat_end-lat_start)*(c_lon-lon_start) - (c_lat-lat_start)*(lon_end-lon_start)).abs

          #compute the AB segment length
          lab = ((lat_end-lat_start)**2 + (lon_end-lon_start)**2).sqrt

          #compute the triangle height
          h = area2/lab

          #if the line intersects the circle
          return false
        elsif zone['boundaries']['shape'] == 'polygon'
          coordinates = zone['boundaries']['coordinates']

          if coordinates.length < 3
            next
          end

          if intersects_polygon?(coordinates, lat_start, lon_start, lat_end, lon_end)
            return false
          end
        end
      end

      # puts "#{(DateTime.now - start_time).to_f.round(5)} seconds finding zones"

      return true
    end

    def parse_coordinate(coord_string)
      return nil if coord_string.blank?

      parts = coord_string.split(/[^\d\w]+/)
      number = (parts[1].to_f + parts[2].to_f/60 + parts[3].to_f/(60*60))
      direction = parts[4].downcase

      if direction == 's' || direction == 'w'
        number *= -1
      end

      number
    end
  end

end