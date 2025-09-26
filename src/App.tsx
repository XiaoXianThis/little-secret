import {Router, Route} from "preact-router"
import IndexPage from "./pages/IndexPage.tsx";
import EncryptPage from "./pages/encrypt/EncryptPage.tsx";
import BrowsePage from "./pages/browse/BrowsePage.tsx";


export function App() {

  return (
    <>
      <Router>
        <Route path="/" component={IndexPage} />
        <Route path="/encrypt" component={EncryptPage} />
        <Route path="/browse" component={BrowsePage} />
      </Router>
    </>
  )
}
