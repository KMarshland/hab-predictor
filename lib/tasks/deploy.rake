namespace :deploy do

  desc 'Creates a package suitable for deployment to aws and cleans old versions'
  task :make => [:bundle, :clean] do

  end

  desc 'Creates a package suitable for deployment to aws'
  task :bundle do
    puts 'Starting zip'
    name = "#{File.basename(Rails.root.to_s)}_#{Time.now.strftime('%Y%m%d-%H-%M-%S')}.zip"

    excluded = %w{
        .git/*
        tmp/*
        .idea/*
        lib/data/*
    }

    `zip ../#{name} -r . #{excluded.map{|x| "-x \"#{x}\""}.join(' ')}`

    puts "Made #{name}"
  end

  desc 'Removes all but the most recent deploy zip'
  task :clean do
    files = deploys
    newer = {
        date: 0
    }
    files.each do |file|
      date = file.gsub(/\D/, '').to_i

      if newer[:date] < date
        safe_remove newer[:file]

        newer = {
            date: date,
            file: file
        }
      else
        safe_remove file
      end
    end
  end

  desc 'Removes all deploy zips'
  task :clobber do
    files = deploys
    files.each do |file|
      safe_remove file
    end
  end

  #removes the given file from the parent directory
  def safe_remove(file)
    return unless file.present?
    puts "Removing #{file}"
    `rm ../#{file}`
  end

  #gives all the filenames that look like a deploy
  def deploys
    filenames lambda {|name|
      name =~ /#{File.basename(Rails.root.to_s)}_.*\.zip$/;
    }
  end

  #the filenames in the parent directory, filtered by the labmda if provided
  def filenames(filter=nil)
    filter = lambda{|name| true } if filter.nil?
    Dir.entries('..').select(&filter)
  end

end