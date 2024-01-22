export function isAddressValid(addr: string) {
    const REGEX = /^[a-z2-7]{56}((-dev-server)|(-dev-client)|())$/g;
    return REGEX.test(addr);
}
