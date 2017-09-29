
module ProcessedDatasets

  class << self

    def on(on_date=nil)
      unless ENV['AZURE_STORAGE_ACCOUNT'].present? && ENV['AZURE_STORAGE_ACCESS_KEY'].present?
        raise 'No Azure Storage Keys provided'
      end

      Azure::Storage.setup(storage_account_name: ENV['AZURE_STORAGE_ACCOUNT'], storage_access_key: ENV['AZURE_STORAGE_ACCESS_KEY'])
      blobs = Azure::Storage::Blob::BlobService.new
      blobs.with_filter(Azure::Storage::Core::Filter::ExponentialRetryPolicyFilter.new)

      prefix = 'gfs'
      if on_date.present?
        "gfs_4_#{on_date.strftime('%Y%m%d')}"
      end

      blobs.list_blobs('data', prefix: prefix).map do |blob|
        blob.name.split('.').first
      end
    end

    def last_dataset
      self.on self.last_date
    end

    def last_date
      on_date = DateTime.now

      until self.on(on_date).count > 0
        on_date -= 1.day
      end

      on_date
    end

    def all
      self.on nil
    end

  end

end
