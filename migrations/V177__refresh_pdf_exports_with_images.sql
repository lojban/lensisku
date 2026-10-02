-- Previously cached PDFs omitted definition images. Rebuild them with the
-- column-bounded image layout on the next request/background refresh.
DELETE FROM cached_dictionary_exports WHERE format = 'pdf';
