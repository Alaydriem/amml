export interface WizardFooterText {
    /** Left note ("The install button may ask for admin permissions..."). */
    note?: string;
    /** Text button before the primary one ("Skip"). Omit for none. */
    secondary?: string;
    primary: string;
}

export interface WizardFooterProps {
    text: WizardFooterText;
    primaryDisabled?: boolean;
    onsecondary?: () => void;
    onprimary?: () => void;
}
