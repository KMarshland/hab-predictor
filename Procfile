web: bundle exec puma -C config/puma.rb -e production -p 3000
worker: bundle exec sidekiq -c 3 -v
redis: redis-server