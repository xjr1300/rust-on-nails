--: User()

--! get_users : User
SELECT
    id,
    email
FROM users;

-- `create_user`クエリを追加
--! create_user
INSERT INTO users (email, hashed_password)
VALUES (:email, :hased_password);
