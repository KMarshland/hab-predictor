FROM buildpack-deps:yakkety

###############
# FROM ruby:2.3, except this version uses our existing docker stack
# Copied from https://github.com/docker-library/ruby/blob/64121ac0a34d07ff1c7341651f8775476cba6c41/2.3/Dockerfile

# skip installing gem documentation
RUN mkdir -p /usr/local/etc \
	&& { \
		echo 'install: --no-document'; \
		echo 'update: --no-document'; \
	} >> /usr/local/etc/gemrc

ENV RUBY_MAJOR 2.3
ENV RUBY_VERSION 2.3.3
ENV RUBY_DOWNLOAD_SHA256 241408c8c555b258846368830a06146e4849a1d58dcaf6b14a3b6a73058115b7
ENV RUBYGEMS_VERSION 2.6.8

# some of ruby's build scripts are written in ruby
#   we purge system ruby later to make sure our final image uses what we just built
RUN set -ex \
	\
	&& buildDeps=' \
		bison \
		libgdbm-dev \
		ruby \
	' \
	&& apt-get update \
	&& apt-get install -y --no-install-recommends $buildDeps \
	&& rm -rf /var/lib/apt/lists/* \
	\
	&& wget -O ruby.tar.gz "https://cache.ruby-lang.org/pub/ruby/${RUBY_MAJOR%-rc}/ruby-$RUBY_VERSION.tar.gz" \
	&& echo "$RUBY_DOWNLOAD_SHA256 *ruby.tar.gz" | sha256sum -c - \
	\
	&& mkdir -p /usr/src/ruby \
	&& tar -xzf ruby.tar.gz -C /usr/src/ruby --strip-components=1 \
	&& rm ruby.tar.gz \
	\
	&& cd /usr/src/ruby \
	\
# hack in "ENABLE_PATH_CHECK" disabling to suppress:
#   warning: Insecure world writable dir
	&& { \
		echo '#define ENABLE_PATH_CHECK 0'; \
		echo; \
		cat file.c; \
	} > file.c.new \
	&& mv file.c.new file.c \
	\
	&& autoconf \
	&& ./configure --disable-install-doc --enable-shared \
	&& make -j"$(nproc)" \
	&& make install \
	\
	&& apt-get purge -y --auto-remove $buildDeps \
	&& cd / \
	&& rm -r /usr/src/ruby \
	\
	&& gem update --system "$RUBYGEMS_VERSION"

ENV BUNDLER_VERSION 1.13.7

RUN gem install bundler --version "$BUNDLER_VERSION"

# install things globally, for great justice
# and don't create ".bundle" in all our apps
ENV GEM_HOME /usr/local/bundle
ENV BUNDLE_PATH="$GEM_HOME" \
	BUNDLE_BIN="$GEM_HOME/bin" \
	BUNDLE_SILENCE_ROOT_WARNING=1 \
	BUNDLE_APP_CONFIG="$GEM_HOME"
ENV PATH $BUNDLE_BIN:$PATH
RUN mkdir -p "$GEM_HOME" "$BUNDLE_BIN" \
	&& chmod 777 "$GEM_HOME" "$BUNDLE_BIN"


############################################################

# This is where our custom docker configuration really begins

# Install basic dependencies
RUN apt-get update -qq
RUN apt-get install -y build-essential nodejs postgresql-client nano redis-server nginx cron less ssh redis-server
ENV TERM xterm

# Install cron
RUN gem update --system 2.6.1
RUN gem install bundler --version $BUNDLER_VERSION

# Set up nginx
RUN rm -rf /etc/nginx/sites-available/default
ADD deploy/nginx.conf.nginx /etc/nginx/nginx.conf

# symlink node
RUN ln -s /usr/bin/nodejs /usr/bin/node

# Navigate to the proper directories
ENV APP_HOME /var/app/current
RUN mkdir -p $APP_HOME
RUN mkdir -p /var/run/puma
WORKDIR $APP_HOME
RUN mkdir -p certs/client

# Create users and groups
RUN groupadd -g 1000 app

RUN useradd -ms /bin/bash nginx
RUN useradd -ms /bin/bash -G app run
RUN useradd -ms /bin/bash -G app deploy

# Set permissions
RUN chown -R deploy:app /usr/local/bundle
RUN chown -R run:app /var/run/puma

RUN mkdir /var/run/nginx
RUN chown nginx /var/log/nginx/error.log
RUN chown -R nginx /var/run/nginx
RUN chmod 777 /var/run/nginx

# Install gems
ADD Gemfile Gemfile
ADD Gemfile.lock Gemfile.lock
RUN bundle install

# Add config files (optimizing cache)
WORKDIR $APP_HOME

RUN mkdir $APP_HOME/config
RUN mkdir $APP_HOME/log

COPY bin $APP_HOME/bin
COPY Rakefile $APP_HOME/Rakefile
COPY config/environments $APP_HOME/config/environments
COPY config/initializers $APP_HOME/config/initializers
COPY config/application.rb config/boot.rb config/cable.yml config/database.yml config/environment.rb config/newrelic.yml config/puma.rb config/puma_prod.rb config/secrets.yml config/sidekiq.yml $APP_HOME/config/

RUN chown -R deploy:app $APP_HOME

RUN nginx -t #Aborts build if nginx config file is invalid
RUN service nginx start

# Update crontabs
COPY config/schedule.rb $APP_HOME/config/schedule.rb
RUN touch /var/log/cron.log
RUN touch /var/log/whenever.log && chmod go+rw /var/log/whenever.log
RUN whenever -w

# Precompile assets
RUN mkdir $APP_HOME/app
RUN mkdir $APP_HOME/vendor

COPY app/assets $APP_HOME/app/assets
COPY lib/assets $APP_HOME/lib/assets
COPY vendor/assets $APP_HOME/vendor/assets

RUN RAILS_ENV=production bundle exec rake assets:precompile

# Add the rest of our sourcecode
ADD . $APP_HOME

# Clear logs (dockerignore doesn't work)
RUN > log/development.log

# Set more permissions
USER root
RUN chmod +x ./container/boot.sh
RUN chmod +x ./container/start_puma.sh
RUN chown -R run:app $APP_HOME


# Stay as root -- the start script will deescalate its own permissions
#USER root
