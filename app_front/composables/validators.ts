export function validateUserName(name: string): string {
    // Test
    if (name.length < 5) return 'Full name must be at least 5 characters long'
    if (name.length > 100) return 'Full name must be at most 100 characters long'
    if (name.trim() !== name) return 'Full name must not start or end with spaces'
    return ''
}

export function validateEmail(email: string): string {
    if (!new RegExp(/^[^\s@]+@[^\s@]+\.[^\s@]+$/).test(email)) return 'Invalid email'
    return ''
}

export function validatePassword(password: string): string {
    // Test
    if (password.length < 8) return 'Password must be at least 8 characters long'
    if (password.length > 100) return 'Password must be at most 100 characters long'
    if (!new RegExp(/^(?=.*[a-z])(?=.*[A-Z])(?=.*[0-9]).*$/).test(password)) return 'Password must contain at least one lowercase letter, one uppercase letter and one digit'
    return ''
}

export function validatePasswordConfirmation(password: string, passwordConfirmation: string): string {
    if (password !== passwordConfirmation) return 'Passwords do not match'
    return ''
}
