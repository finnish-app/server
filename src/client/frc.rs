pub fn validate_frc(frc_captcha_solution: &str) -> bool {
    if frc_captcha_solution == ".UNFINISHED" || frc_captcha_solution == ".ERROR" {
        return false;
    }
    return true;
}
