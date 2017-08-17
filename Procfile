web: bundle exec puma -C config/puma.rb -e production
worker: bundle exec sidekiq -c 3 -v
redis: redis-server
secondaryboot: bash deploy/download_daemon.sh