import FormMain from '../components/FormMain'
import MenuMain from '../components/MenuMain'
import FigureTier from '../components/FigureTier'
import ButtonMain from '../components/ButtonMain'

const address = `HN7cABqLq46Es1jh92dQQisAq662SmxELLLsHHe4YWrH`
const displayedAddress = `${address.slice(0, 4)}…${address.slice(-4)}`

const Claim = () => {
  return (
    <>
      <MenuMain current="claiming" />
      <FormMain>
        <FigureTier className="offset-block-start">Tier 1</FigureTier>
        <p>
          Congrats! <button
          type="button"
          className="ghost"
          title="Connect another wallet"
        >
          <u>{displayedAddress}</u>
        </button>
          is eligible to <data value={17_842.36} className="amount">
          17,842.36 $XENO
        </data>
        </p>
        <ButtonMain>Claim now</ButtonMain>
      </FormMain>
      <button type="button" className="ghost">
        <u>Connect another wallet</u>
      </button>
    </>
  )
}
export default Claim


// <style>
//     .offset-block-start {
//   margin - block - start: -6cap;
// }
//
//     .amount {
//     display: block;
//
//     color: #20ff49;
//
//     text-shadow: 0px 0px 10px rgba(32, 255, 73, 0.5);
//     font-size: 1.875rem;
//     font-weight: 600;
//     line-height: 2em;
// }
//
//     button[type="button"] {
//   margin - inline: auto;
//     color: color-mix(var(--color-mix), currentColor, 15% transparent);
//
//     /* reset */
//     inline-size: fit-content;
//     padding: unset;
//     background-color: unset;
//     font-weight: 200;
//
//     &:not([disabled]):hover {
//     color: currentColor;
// }
//
//     &[disabled]:hover {
//     scale: 1;
//     cursor: not-allowed;
// }
// }
// </style>