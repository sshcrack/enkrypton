export function isAddressValid(addr: string) {
    const REGEX = /^[a-z2-7]{56}((-server)|(-client)|())$/g;
    return REGEX.test(addr);
}
