FROM ruby:2.4

# This is where our custom docker configuration really begins

# Install basic dependencies
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
    build-essential \
    nodejs \
    postgresql-client \
    nano \
    redis-server \
    nginx \
    cron \
    less \
    redis-server \
    cmake
ENV TERM xterm

# Install rust
ENV RUST_VERSION=1.17.0
RUN curl -sO https://static.rust-lang.org/dist/rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz && \
      tar -xzf rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz && \
      ./rust-$RUST_VERSION-x86_64-unknown-linux-gnu/install.sh --without=rust-docs && \
      rm -rf \
        rust-$RUST_VERSION-x86_64-unknown-linux-gnu \
        rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz \
        /var/lib/apt/lists/* \
        /tmp/* \
        /var/tmp/*

# Install cron
RUN gem update --system 2.6.1
RUN gem install bundler --version $BUNDLER_VERSION

# Install grib api
RUN curl 'https://software.ecmwf.int/wiki/download/attachments/3473437/grib_api-1.22.0-Source.tar.gz?api=v2' > grib.tar.gz && \
    mkdir grib && \
    tar -xzf grib.tar.gz -C grib --strip-components 1 && \
    rm grib.tar.gz && \
    mkdir build && \
    cd build && \
    cmake ../grib -DENABLE_GRIB_THREADS=ON -DENABLE_FORTRAN=OFF -DENABLE_PYTHON=OFF -DENABLE_JPG=OFF -DENABLE_NETCDF=OFF && \
    make && make install && \
    cd .. && \
    rm -rf grib && \
    rm -rf build

# symlink node
RUN ln -s /usr/bin/nodejs /usr/bin/node

# Navigate to the proper directories
ENV APP_HOME /var/app/current
RUN mkdir -p $APP_HOME
RUN mkdir -p /var/run/puma
WORKDIR $APP_HOME

# Create users and groups
RUN groupadd -g 1000 app

RUN useradd -ms /bin/bash -G app run
RUN useradd -ms /bin/bash -G app deploy

# Set permissions
RUN chown -R deploy:app /usr/local/bundle
RUN chown -R run:app /var/run/puma

EXPOSE 80

# Add config files (optimizing cache)
WORKDIR $APP_HOME

RUN mkdir $APP_HOME/config
RUN mkdir $APP_HOME/log

COPY bin $APP_HOME/bin
COPY Rakefile $APP_HOME/Rakefile
COPY config/environments $APP_HOME/config/environments
COPY config/initializers $APP_HOME/config/initializers
COPY config/application.rb config/boot.rb config/cable.yml config/database.yml config/environment.rb config/puma.rb config/secrets.yml $APP_HOME/config/

RUN chown -R deploy:app $APP_HOME

# Install gems
ADD Gemfile Gemfile
ADD Gemfile.lock Gemfile.lock
ADD crates crates
RUN bundle install

# Add the rest of our sourcecode
ADD . $APP_HOME

# Build our rust code
RUN bundle exec rake build

# RUN RAILS_ENV=production bundle exec rake assets:precompile

# Set more permissions
USER root
RUN chmod +x ./deploy/start_foreman.sh && \
    chmod +x ./deploy/download_daemon.sh && \
    chown -R run:app $APP_HOME


USER run

CMD ./deploy/start_foreman.sh
