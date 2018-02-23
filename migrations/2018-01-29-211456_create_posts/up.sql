CREATE TABLE listings (
  id int NOT NULL AUTO_INCREMENT,
  domain varchar(255) NOT NULL,
  PRIMARY KEY (id),
  UNIQUE (domain)
);
CREATE TABLE requests (
  id int NOT NULL AUTO_INCREMENT,
  publisher varchar(255) NOT NULL,
  userquality int NOT NULL,
  PRIMARY KEY (id)
);
CREATE TABLE responses (
  id int NOT NULL AUTO_INCREMENT,
  publisher varchar(255) NOT NULL,
  PRIMARY KEY (id)
);