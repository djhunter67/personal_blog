pub mod login;
pub mod passworder;
pub mod session;

/// This module tests the encryption, salt, and peppering of passwords
const PEPPER: [u8; 12] = *b"the_pepperer";

// Tests for the PassWorder struct
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use base64::{Engine, engine::general_purpose};
    use chacha20::{ChaCha20, KeyIvInit, cipher::StreamCipher};

    use crate::security::{login::LoginChecker, passworder::PassWorder};

    use super::*;

    #[test]
    fn test_password_encryption() {
        let pw = PassWorder::new("my_secret_password".to_string());
        let encrypted_pw = pw.encrypt();
        assert_ne!(encrypted_pw.get(), "my_secret_password");
    }

    #[test]
    fn test_password_peppering() {
        let pw = PassWorder::new("my_secret_password".to_string());
        let peppered_pw = pw.pepper();
        assert!(
            peppered_pw
                .get()
                .ends_with(general_purpose::STANDARD.encode(PEPPER).as_str())
        );
    }

    #[test]
    fn test_password_deconstruction() {
        let pw = PassWorder::new("my_secret_password".to_string());
        let salted_peppered_pw = pw.salt().pepper();
        let (salt, hash, pepper) = salted_peppered_pw.deconstruct();
        assert!(!salt.is_empty());
        assert!(!hash.is_empty());
        assert_eq!(
            pepper,
            PEPPER.iter().map(|b| *b as char).collect::<String>()
        );
    }

    #[test]
    fn test_pw_encryption_length() {
        let pw = PassWorder::new("my_secret_password".to_string());
        let encrypted_pw = pw.encrypt();
        assert_eq!(encrypted_pw.get().len(), 36);
    }

    #[test]
    fn test_pw_salt_uniqueness() {
        let pw1 = PassWorder::new("my_secret_password".to_string());
        let salted_pw1 = pw1.salt();
        let pw2 = PassWorder::new("my_secret_password".to_string());
        let salted_pw2 = pw2.salt();
        assert_ne!(salted_pw1.get(), salted_pw2.get());
    }

    #[test]
    fn test_pw_pepper_consistency() {
        let pw1 = PassWorder::new("my_secret_password".to_string());
        let pw2 = PassWorder::new("my_secret_password".to_string());
        let peppered_pw1 = pw1.pepper();
        let peppered_pw2 = pw2.pepper();
        assert_eq!(peppered_pw1.get(), peppered_pw2.get());
    }

    #[test]
    fn test_pw_contains_dollar_sign() {
        let pw = PassWorder::new("my_secret_password".to_string());
        let salted_pw = pw.encrypt().salt().pepper();
        assert!(salted_pw.get().contains('$'));
    }

    #[test]
    fn test_random_is_lenght_before_and_after_conversion() {
        let random_salt: [u8; 16] = rand::random();
        let key: [u8; 32] = *b"an example very very secret key!";
        let nonce: [u8; 12] = *b"unique nonce";

        let mut encryptor = ChaCha20::new(&key.into(), &nonce.into());

        let mut cipher_text = random_salt;

        encryptor.apply_keystream(&mut cipher_text);

        assert_eq!(random_salt.len(), cipher_text.len());
    }

    #[test]
    fn test_login_checker_verfier() {
        let mut verifier: LoginChecker =
            LoginChecker::new("the_email".to_string(), "the_password".to_string());

        let user_pw = "the_password".to_string();

        let encrypted_pw: PassWorder = PassWorder::new(verifier.get_pw().clone())
            .encrypt()
            .salt()
            .pepper();

        let (_, pw, _) = encrypted_pw.deconstruct();

        verifier.set_pw(pw);

        assert!(verifier.pw_verify(user_pw));
    }

    #[test]
    fn test_login_verifier_failure() {
        let mut verifier: LoginChecker =
            LoginChecker::new("the_email".to_string(), "the_password".to_string());

        let user_pw = "wrong_password".to_string();

        let encrypted_pw: PassWorder = PassWorder::new(verifier.get_pw().clone())
            .encrypt()
            .salt()
            .pepper();

        let (_, pw, _) = encrypted_pw.deconstruct();

        verifier.set_pw(pw);

        assert!(!verifier.pw_verify(user_pw));
    }

    #[test]
    fn test_login_verifier_with_salt_and_pepper() {
        let mut verifier: LoginChecker =
            LoginChecker::new("the_email".to_string(), "the_password".to_string());

        let user_pw = "the_password".to_string();

        let encrypted_pw: PassWorder = PassWorder::new(verifier.get_pw().clone())
            .encrypt()
            .salt()
            .pepper();

        let (_, pw, _) = encrypted_pw.deconstruct();

        verifier.set_pw(pw);

        assert!(verifier.pw_verify(user_pw));
    }

    #[test]
    fn test_login_verifier_with_incorrect_password() {
        let mut verifier: LoginChecker =
            LoginChecker::new("the_email".to_string(), "the_password".to_string());

        let user_pw = "incorrect_password".to_string();

        let encrypted_pw: PassWorder = PassWorder::new(verifier.get_pw().clone())
            .encrypt()
            .salt()
            .pepper();

        let (_, pw, _) = encrypted_pw.deconstruct();

        verifier.set_pw(pw);

        assert!(!verifier.pw_verify(user_pw));
    }

    #[test]
    fn test_login_verifier_with_empty_password() {
        let mut verifier: LoginChecker =
            LoginChecker::new("the_email".to_string(), "the_password".to_string());

        let user_pw = String::new();

        let encrypted_pw: PassWorder = PassWorder::new(verifier.get_pw().clone())
            .encrypt()
            .salt()
            .pepper();

        let (_, pw, _) = encrypted_pw.deconstruct();

        verifier.set_pw(pw);

        assert!(!verifier.pw_verify(user_pw));
    }

    #[test]
    fn test_login_verifier_with_special_characters() {
        let mut verifier: LoginChecker =
            LoginChecker::new("the_email".to_string(), "p@$$w0rd!".to_string());

        let user_pw = "p@$$w0rd!".to_string();

        let encrypted_pw: PassWorder = PassWorder::new(verifier.get_pw().clone())
            .encrypt()
            .salt()
            .pepper();

        let (_, pw, _) = encrypted_pw.deconstruct();

        verifier.set_pw(pw);

        assert!(verifier.pw_verify(user_pw));
    }

    #[test]
    fn test_login_verifier_with_long_password() {
        let mut verifier: LoginChecker = LoginChecker::new(
            "the_email".to_string(),
            "a_very_long_password_that_exceeds_normal_length".to_string(),
        );

        let user_pw = "a_very_long_password_that_exceeds_normal_length".to_string();

        let encrypted_pw: PassWorder = PassWorder::new(verifier.get_pw().clone())
            .encrypt()
            .salt()
            .pepper();

        let (_, pw, _) = encrypted_pw.deconstruct();

        verifier.set_pw(pw);

        assert!(verifier.pw_verify(user_pw));
    }

    #[test]
    fn test_passwords_with_spaces_and_tabs() {
        let mut verifier: LoginChecker = LoginChecker::new(
            "the_email".to_string(),
            "   password_with_spaces_and_tabs\t".to_string(),
        );

        let user_pw = "   password_with_spaces_and_tabs\t".to_string();

        let encrypted_pw: PassWorder = PassWorder::new(verifier.get_pw().clone())
            .encrypt()
            .salt()
            .pepper();

        let (_, pw, _) = encrypted_pw.deconstruct();

        verifier.set_pw(pw);

        assert!(verifier.pw_verify(user_pw));
    }

    #[test]
    fn test_passwords_with_spaces_and_tabs_2() {
        let mut verifier: LoginChecker = LoginChecker::new(
            "the_email".to_string(),
            "   password with spaces and tabs\t".to_string(),
        );

        let user_pw = "   password with spaces and tabs\t".to_string();

        let encrypted_pw: PassWorder = PassWorder::new(verifier.get_pw().clone())
            .encrypt()
            .salt()
            .pepper();

        let (_, pw, _) = encrypted_pw.deconstruct();

        verifier.set_pw(pw);

        assert!(verifier.pw_verify(user_pw));
    }
}
