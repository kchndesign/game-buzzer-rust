import { Button, FormControl, FormLabel, Input } from '@mui/joy';
import { FunctionComponent, useState } from 'react';
import MaxWidthCard from '../styled-components/MaxWidthCard';
import { useNavigate } from 'react-router-dom';

interface PlayerCodeInputProps {}

const PlayerCodeInput: FunctionComponent<PlayerCodeInputProps> = () => {
    const [input, setInput] = useState<string>();
    const [formMessage, setFormMessage] = useState<string | undefined>();
    const navigator = useNavigate();

    const handlePlayButton = () => {
        setFormMessage(undefined);

        if (!input) {
            setFormMessage('Please input code to play');
            return;
        }

        navigator(`/${input}`);
    };

    return (
        <MaxWidthCard>
            <FormControl className="mb-4">
                <FormLabel>Input game code</FormLabel>
                <Input
                    required
                    type="text"
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                />
            </FormControl>

            {formMessage ? <p>{formMessage}</p> : null}

            <Button
                className="d-block ms-auto"
                onClick={handlePlayButton}
            >
                Play
            </Button>
        </MaxWidthCard>
    );
};

export default PlayerCodeInput;
