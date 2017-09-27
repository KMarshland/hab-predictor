require 'zip'
require 'fileutils'

# downloads preprocessed data from Azure
class ImportWorker

  def perform(dataset_url)
    @folder_name = dataset_url.split('/').last.split('.').first
    @filename = "#{@folder_name}.zip"
    @zip_path = Rails.root.join('data', @filename)

    @folder_path = Rails.root.join('data', @folder_name)
    @partial_folder_path = Rails.root.join('data', "#{@folder_name}.partial")

    puts "Importing #{@folder_name}.zip"

    # remove any old data
    FileUtils.remove_dir @folder_path if Dir.exist? @folder_path
    FileUtils.remove_dir @partial_folder_path if Dir.exist? @partial_folder_path

    download
    unzip
    clean
  end

  def download
    puts 'Downloading'

    unless ENV['AZURE_STORAGE_ACCOUNT'].present? && ENV['AZURE_STORAGE_ACCESS_KEY'].present?
      raise 'No Azure Storage Keys provided'
    end

    Azure::Storage.setup(storage_account_name: ENV['AZURE_STORAGE_ACCOUNT'], storage_access_key: ENV['AZURE_STORAGE_ACCESS_KEY'])
    blobs = Azure::Storage::Blob::BlobService.new
    blobs.with_filter(Azure::Storage::Core::Filter::ExponentialRetryPolicyFilter.new)


    _blob, content = blobs.get_blob('data', @filename)
    ::File.open(@zip_path, 'wb') {|f| f.write(content)}
  end

  def unzip

    puts 'Unzipping'

    Zip::File.open(@zip_path) do |zip_file|
      # Handle entries one by one
      zip_file.each do |entry|

        entry_folder = entry.name.split('/')
        entry_folder = entry_folder.take(entry_folder.size - 1).join('/')

        FileUtils::mkdir_p @partial_folder_path.join(entry_folder)

        entry.extract(@partial_folder_path.join(entry.name))

      end
    end
  end

  def clean
    puts 'Cleaning'

    # Remove zip
    File.delete(@zip_path) if File.exists? @zip_path

    # rename to non-partial
    FileUtils.mv @partial_folder_path, @folder_path
  end

end
