module Prediction

  TAWHIRI_DATETIME_FORMAT = '%Y-%m-%dT%H:%M:%SZ'

  class << self

    def socket(possible_errors=1)
      return @socket unless @socket.nil?

      # server (habmc_interface.py) must be running
      # server should be started automatically by the procfile

      # Connects to Tawhiri using UNIX domain socket
      begin
        @socket = UNIXSocket.new(File.join(Rails.root, 'lib', 'prediction', 'sock'))
      rescue
        if possible_errors == 0
          raise 'ERROR: Connection failed. Make sure the server is running.'.red
        else
          start_server
          return socket(possible_errors - 1)
        end
      end

      @socket
    end

    def close_conn
      if @socket.present?
        @socket.close
        @socket = nil
      end
    end

    def predict(params, is_guidance:true, include_metadata:false)

      errors = []
      get_param = lambda {|name|
        r = params[name]

        r = nil if r.blank? || r == 'undefined'
        errors << "Missing required parameter #{name}" if r.nil?

        r
      }

      lat = get_param.call(:lat).to_f
      lon = get_param.call(:lon).to_f
      altitude = get_param.call(:altitude).to_f

      launch_time = get_param.call(:time)
      duration = get_param.call(:duration).to_f

      ascent_rate = get_param.call(:ascent_rate).to_f
      descent_rate = get_param.call(:descent_rate).to_f
      burst_altitude = get_param.call(:burst_altitude).to_f

      if errors.present?
        return {success: false, errors: errors}
      end

      launch_time = launch_time.to_datetime
      stop_time = (launch_time + duration.minutes).to_datetime

      #make the request
      request_params = {
          profile: params[:profile] || 'float_profile',
          launch_latitude: lat,
          launch_longitude: lon % 360, # convert to positive angle for tawhiri API
          launch_altitude: altitude - 1,
          float_altitude: altitude,
          launch_datetime: launch_time.utc.strftime(TAWHIRI_DATETIME_FORMAT), #Warning: their API sucks at parsing times
          stop_datetime: stop_time.utc.strftime(TAWHIRI_DATETIME_FORMAT),
          ascent_rate: ascent_rate,
          burst_altitude: burst_altitude,
          descent_rate: descent_rate,
          is_guidance: is_guidance # For internal use by habmc_interface.py, not passed to API
      }

      result = make_request(request_params, is_guidance: is_guidance, include_metadata: include_metadata)

      result.map!{|a|
        a['longitude'] = ((a['longitude'] + 180) % 360) - 180 # convert angle back to negative
        a
      } unless result.is_a? Hash

      result
    end

    def make_request(request_params, is_guidance:true, retries:2, include_metadata:false)
      # SEE http://tawhiri.cusf.co.uk/en/latest/api.html for API documentation
      # https://www.ncdc.noaa.gov/data-access/model-data/model-datasets/global-forcast-system-gfs for data source
      # habmc_interface.py is the other end of the socket

      prediction = nil
      if has_dataset_for?(DateTime.strptime(request_params[:launch_datetime], TAWHIRI_DATETIME_FORMAT))
        begin
          request_params[:include_metadata] = include_metadata
          request = socket.send JSON(request_params), 0
          response = socket.recv(524288)
          raise 'No response from server' if response.blank?

          prediction = JSON(response)
        rescue
          unless retries == 0
            sleep 1 #wait for servers to reset
            return make_request(request_params, is_guidance:is_guidance, include_metadata: include_metadata, retries: retries-1)
          end

          return {
              success: false,
              errors: ["#{$!}"]
          }
        end

      else
        prediction = make_http_request(request_params)

        prediction = prediction['prediction'] unless include_metadata

        prediction['metadata']['used_api'] = true if include_metadata
      end

      return {
          success: false,
          errors: ['No response']
      } if prediction.blank?

      # prediction receives and uses extra data that guidance does not
      unless is_guidance || include_metadata
        prediction = [*prediction[0]['trajectory'], *prediction[1]['trajectory']]
      end

      prediction
    end

    def make_http_request(request_params, params={})
      #SEE http://tawhiri.cusf.co.uk/en/latest/api.html for documentation
      #    https://www.ncdc.noaa.gov/data-access/model-data/model-datasets/global-forcast-system-gfs for data source
      url = "http://predict.cusf.co.uk/api/v1/?#{request_params.collect { |k,v| "#{k}=#{(v.to_s)}" }.join('&')}"
      response = http_get url

      unless response.methods.include?(:code) && response.code.to_i == 200
        if response.methods.include?(:code)
          errors = [
              "HTTP #{response.code}",
              JSON(response.body)
          ]
        else
          errors = [
              response['body']
          ]
        end

        return {
            success: false,
            errors: errors,
            request: url
        }
      end

      # ascending = params.with_indifferent_access[:include_ascent]
      # duration = params.with_indifferent_access[:duration]
      #
      # parse the result
      # start = request_params.with_indifferent_access[:launch_datetime].to_datetime.to_i
      # prediction = JSON(response.body).with_indifferent_access[:prediction][ascending ? 0 : 1][:trajectory].select{|p|
      #   p[:datetime].to_datetime.to_i - start < duration * 60
      # }
      #
      # prediction

      JSON(response.body)
    end

    def http_get(url, headers={}, read_timeout=60)

      uri = URI.parse(URI.encode(url))
      http = Net::HTTP.new(uri.host, uri.port)
      http.use_ssl = true if uri.scheme == 'https'
      http.read_timeout = read_timeout
      response = nil
      begin
        http.start do
          request = Net::HTTP::Get.new(uri.request_uri, headers)
          response = http.request(request)
        end
      rescue Timeout::Error => e
        response = {'body' => 'Timeout error'}
      rescue StandardError => e
        response = {'body' => "Error while making request: #{e}"}
      end

      return response
    end

    def has_dataset_for?(datetime)
      return false if self.datasets.blank?
      (self.datasets.first - 12.hours) < datetime && datetime < (self.datasets.last + 12.hours)
    end

    def datasets
      sets = []
      #filenames are of the form yyyymmddhh
      Dir.glob("/srv/tawhiri-datasets/*").each do |filename|
        # partially downloaded files have names download-%Y%m%d%H
        # fully downloaded ones have names %Y%m%d%H
        num_date = File.basename(filename).to_i

        sets << DateTime.strptime(num_date.to_s, '%Y%m%d%H') if num_date > 0
      end

      sets.sort
    end

  end
end

require_relative './active_guidance'
