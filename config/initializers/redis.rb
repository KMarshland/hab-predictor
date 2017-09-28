
if ENV['REDIS_URL'].present?
  uri = URI.parse(ENV['REDIS_URL'])
  $redis = Redis.new(host: uri.host, port: uri.port, password: uri.password)
else
  $redis = Redis.new(host: 'localhost', port: 6379)
end
