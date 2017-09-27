
class UploadWorker

  def perform(dataset_url)
    folder_name = dataset_url.split('/').last.split('.').first
    puts "Uploading #{folder_name}.zip"

    unless ENV['AZURE_STORAGE_ACCOUNT'].present? && ENV['AZURE_STORAGE_ACCESS_KEY'].present?
      raise 'No Azure Storage Keys provided'
    end

    Azure::Storage.setup(storage_account_name: ENV['AZURE_STORAGE_ACCOUNT'], storage_access_key: ENV['AZURE_STORAGE_ACCESS_KEY'])
    blobs = Azure::Storage::Blob::BlobService.new
    blobs.with_filter(Azure::Storage::Core::Filter::ExponentialRetryPolicyFilter.new)

    output_file = Rails.root.join('data', "#{folder_name}.zip")
    content = ::File.open(output_file, 'rb') { |file| file.read }
    blobs.create_block_blob('data', "#{folder_name}.zip", content).inspect

  end

end
