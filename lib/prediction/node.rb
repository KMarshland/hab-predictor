
module Prediction
  class << self
    #how frequently to check an altitude change, in minutes; higher is lower res
    def time_variance; 180 end

    #how much to vary the altitude, in meters; higher is lower res
    def altitude_resolution; 500 end

    #how far to vary the altitude each step; higher leads to greater altitude changes
    def altitude_variance; 5 end
  end

  class Node
    attr_accessor :lat, :lon, :altitude, :time, :visited, :previous, :movement_cost, :prediction, :parent, :cost, :child_cost, :generation

    def initialize(lat, lon, altitude, time, visited=false, previous=nil, movement_cost=nil, prediction=nil, parent=nil, cost=nil, child_cost=nil, generation=nil)
      tolerance = 6
      @lat = lat.to_f.round(tolerance)
      @lon = lon.to_f.round(tolerance)
      @altitude = altitude.to_f.round
      @time = time
      @visited = visited
      @previous = previous
      @movement_cost = movement_cost
      @prediction = prediction
      @parent = parent
      @cost = cost
      @child_cost = child_cost
      @generation = generation

      Node.add_node(self)

      Thread.new {
        # fuck you mutexes
        # self.prediction
      }
    end


    def hash
      [self.lat, self.lon, self.altitude, self.time].hash
    end

    def eql?(other)
      self.hash == other.hash
    end

    # get the node's total cost by adding the heuristic cost to the generation cost
    def total_cost
      @cost + @child_cost
    end

    def start_parallelization(repeats)
      return if repeats < 0

      Thread.new {
        self.neighbors(lambda{|node|
          true
        }).each do |node|
          node.start_parallelization(repeats - 1)
        end
      }
    end

    #approximates whether or not you're in an faa zone with a straight line
    def faa_zones_approx(parent)
      # start_time = DateTime.now
      return Prediction::find_faa_zones(self.lat, self.lon, parent.lat, parent.lon)
    end

    #returns all the faa zones that it crosses through
    def faa_zones(inverse_resolution=1)
      start_time = DateTime.now

      prediction = self.prediction
      zones = []
      prediction.each_with_index do |predict, i|
        next unless i % inverse_resolution == 0
        zones.concat Prediction::find_faa_zones(predict['latitude'], predict['longitude'])
      end

      zones.uniq!

      puts "#{(DateTime.now - start_time).to_f.round(5)} seconds elapsed finding faa zones"

      zones
    end

    # checks if the path passes through any restricted countries
    def countries(parent)
      return Prediction::find_countries(self.lat, self.lon, parent.lat, parent.lon)
    end

    # Note: guidance makes hundreds of predictions per second, so any slow code will bottleneck things
    # Each prediction takes around 5 milliseconds, assuming standard time_variance
    def prediction
      return @prediction unless @prediction.nil?
      # puts "Making prediction from #{@lat}, #{@lon} at #{@altitude}m"

      @prediction = Prediction::predict({
                                            lat: @lat,
                                            lon: @lon,
                                            altitude: @altitude,

                                            time: @time,
                                            duration: Prediction::time_variance,

                                            ascent_rate: 0.02,
                                            descent_rate: 0.02,
                                            burst_altitude: 24000
                                        })

      if prediction.is_a?(Hash) && prediction.has_key?(:success) && !prediction[:success]
        puts 'Error making prediction: '.red
        puts prediction[:errors]
      end

      @prediction
    end

    def neighbors(filter)
      prediction = self.prediction

      neighbors = []

      if prediction.is_a?(Hash) && prediction.has_key?(:success) && !prediction[:success]
        puts 'Error making prediction: '.red
        puts prediction[:errors]
      else
        last = prediction.last.with_indifferent_access

        lat = last[:latitude].to_f
        lon = last[:longitude].to_f
        start_alt = last[:altitude].to_f
        time = last[:datetime]

        #vary the altitudes
        (-Prediction::altitude_variance..Prediction::altitude_variance).each do |v|
          new_altitude = v*Prediction::altitude_resolution + start_alt
          new_node = Node.from_pos(lat, lon, new_altitude, time)
          new_node.parent = self
          neighbors.push(new_node)
        end

        neighbors.select!{|neigh|
          filter.call(neigh, self)
        }
      end


      neighbors
    end

    def build_chain
      current = self
      result = []
      while current.present?
        break if result.include? current
        result << current
        current = current.previous
      end

      result.reverse
    end

    def ==(y)
      tolerance = 0.01
      (self.lat - y.lat).abs < tolerance &&
          (self.lon - y.lon).abs < tolerance &&
          (self.altitude - y.altitude).abs < 10
    end

    def as_json(options={})
      self.to_json
    end

    def to_json(options={})
      {lat: self.lat, lon: self.lon, altitude: self.altitude, time: self.time, prediction: self.prediction}
    end

    def to_string
      Node.to_string(self.lat, self.lon, self.altitude, self.time)
    end

    def self.to_string(lat, lon, altitude, time)
      coordinates_tolerance = 5
      altitude_tolerance = -1
      "#{lat.round(coordinates_tolerance)}|#{lon.round(coordinates_tolerance)}|#{altitude.round(altitude_tolerance)}|#{time}"
    end

    @@nodes = {}
    def self.from_pos(lat, lon, altitude, time)
      hash = [lat, lon, altitude, time].hash
      unless @@nodes.has_key? hash
        @@nodes[hash] = Node.new(lat, lon, altitude, time)
      end

      @@nodes[hash]
    end

    def self.add_node(node)
      @@nodes[node.hash] = node
    end
  end
end
