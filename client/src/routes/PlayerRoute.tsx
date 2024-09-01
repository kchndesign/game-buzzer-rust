import { FunctionComponent } from 'react';
import { useParams, useRoutes } from 'react-router-dom';

interface PlayerRouteProps {}

const PlayerRoute: FunctionComponent<PlayerRouteProps> = () => {
    const params = useParams();

    return <div>PlayerRoute {params.code} </div>;
};

export default PlayerRoute;
