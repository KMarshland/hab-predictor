
namespace :standalone_downloader do

  task :build => :environment do

    require Rails.root.join('lib', 'downloader', 'standalone_builder.rb')

    Downloader::Standalone.build

  end

end
