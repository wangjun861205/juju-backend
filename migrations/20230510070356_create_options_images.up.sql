-- Add up migration script here
CREATE TABLE options_images (
    id SERIAL NOT NULL PRIMARY KEY, 
    option_id INT NOT NULL, 
    uploaded_file_id INT NOT NULL, 
    CONSTRAINT fk_option_id FOREIGN KEY (option_id) REFERENCES options (id) ON DELETE CASCADE, 
    CONSTRAINT fk_uploaded_file_id FOREIGN KEY (uploaded_file_id) REFERENCES uploaded_files (id) ON DELETE CASCADE,
    CONSTRAINT uni_option_id_uploaded_file_id UNIQUE (option_id, uploaded_file_id)
);
