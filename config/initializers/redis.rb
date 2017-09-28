
if ENV['REDIS_URL'].present?
  uri = URI.parse(ENV['REDIS_URL'])
  $redis = Redis.new(host: uri.host, port: uri.port, user: uri.user, password: uri.password, ssl: 'required', ssl_params: {
      verify_mode: OpenSSL::SSL::VERIFY_NONE
  })
else
  $redis = Redis.new(host: 'localhost', port: 6379)
end
